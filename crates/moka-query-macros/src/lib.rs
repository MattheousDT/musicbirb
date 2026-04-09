extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
	parse_macro_input, Expr, ExprArray, ExprLit, FnArg, ItemTrait, Lit, LitStr, Meta, ReturnType, TraitItem, Type,
};

fn strip_reference(ty: &Type) -> Type {
	if let Type::Reference(type_ref) = ty {
		*type_ref.elem.clone()
	} else {
		ty.clone()
	}
}

fn extract_ok_type(ret: &ReturnType) -> Option<Type> {
	if let ReturnType::Type(_, ty) = ret {
		if let Type::Path(path) = &**ty {
			let segment = path.path.segments.last()?;
			if segment.ident == "Result" {
				if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
					if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
						return Some(inner_ty.clone());
					}
				}
			}
		}
	}
	None
}

#[proc_macro_attribute]
pub fn moka_query_proxy(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut input_trait = parse_macro_input!(item as ItemTrait);

	let mut namespace = String::new();
	let meta_parser = syn::meta::parser(|meta| {
		if meta.path.is_ident("namespace") {
			let lit: LitStr = meta.value()?.parse()?;
			namespace = lit.value();
			Ok(())
		} else {
			Err(meta.error("unsupported property"))
		}
	});
	parse_macro_input!(attr with meta_parser);

	let trait_name = &input_trait.ident;
	let proxy_name = format_ident!("Cached{}", trait_name);
	let vis = &input_trait.vis;

	let mut generated_types = Vec::new();
	let mut trait_impl_methods = Vec::new();
	let mut trait_observe_defs = Vec::new();
	let mut ffi_global_functions = Vec::new();

	for item in &mut input_trait.items {
		if let TraitItem::Fn(method) = item {
			let sig = &method.sig;
			let method_name = &sig.ident;

			if method_name.to_string().starts_with("uniffi_") {
				continue;
			}

			let inputs = &sig.inputs;
			let output = &sig.output;
			let is_async = sig.asyncness.is_some();

			let mut is_query = false;
			let mut is_mutation = false;
			let mut query_key_template = String::new();
			let mut invalidates_patterns = Vec::new();

			for attr in &method.attrs {
				if attr.path().is_ident("query") {
					is_query = true;
					if let Meta::List(meta) = &attr.meta {
						let tokens = meta.tokens.clone();
						let expr: syn::ExprAssign = syn::parse2(tokens).expect("Expected `key = \"...\"`");
						if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = *expr.right {
							query_key_template = lit_str.value();
						}
					}
				} else if attr.path().is_ident("mutation") {
					is_mutation = true;
					if let Meta::List(meta) = &attr.meta {
						let tokens = meta.tokens.clone();
						let expr: syn::ExprAssign = syn::parse2(tokens).expect("Expected `invalidates = [...]`");
						if let Expr::Array(ExprArray { elems, .. }) = *expr.right {
							for elem in elems {
								if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = elem {
									invalidates_patterns.push(lit_str.value());
								}
							}
						}
					}
				}
			}

			method.attrs.retain(|attr| !attr.path().is_ident("query") && !attr.path().is_ident("mutation"));

			let mut arg_names = Vec::new();
			let mut arg_types_owned = Vec::new();
			let mut arg_is_ref = Vec::new();

			for arg in inputs.iter().skip(1) {
				if let FnArg::Typed(pat_type) = arg {
					if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
						arg_names.push(&pat_ident.ident);
						arg_types_owned.push(strip_reference(&pat_type.ty));
						arg_is_ref.push(matches!(*pat_type.ty, Type::Reference(_)));
					}
				}
			}

			let generics = &sig.generics;

			if is_mutation {
				trait_impl_methods.push(quote! {
					async fn #method_name #generics (#inputs) #output {
						let res = self.inner.#method_name(#(#arg_names),*).await;
						if res.is_ok() {
							#( self.global_client.invalidate_pattern(#invalidates_patterns).await; )*
						}
						res
					}
				});
			} else if is_async {
				trait_impl_methods.push(quote! {
					async fn #method_name #generics (#inputs) #output {
						self.inner.#method_name(#(#arg_names),*).await
					}
				});
			} else {
				trait_impl_methods.push(quote! {
					fn #method_name #generics (#inputs) #output {
						self.inner.#method_name(#(#arg_names),*)
					}
				});
			}

			if is_query {
				let inner_ok_type = extract_ok_type(output).expect("Query methods must return a Result<T, E>");
				let observe_method_name = format_ident!("observe_{}", method_name);
				let full_key = format!("{}/{}", namespace, query_key_template);

				let method_camel = method_name
					.to_string()
					.split('_')
					.map(|s| format!("{}{}", &s[..1].to_uppercase(), &s[1..]))
					.collect::<String>();

				let stream_struct_name = format_ident!("Observe{}Stream", method_camel);
				let state_enum_name = format_ident!("Observe{}State", method_camel);

				let trait_prefix = trait_name.to_string().replace("Provider", "");
				let ffi_func_name = format_ident!("observe_{}_{}", trait_prefix.to_lowercase(), method_name);

				let captures: Vec<_> = arg_names
					.iter()
					.zip(arg_is_ref.iter())
					.map(|(name, is_ref)| {
						if *is_ref {
							quote! { let #name = #name.clone(); let #name = &#name; }
						} else {
							quote! { let #name = #name.clone(); }
						}
					})
					.collect();

				trait_observe_defs.push(quote! {
					#[cfg(not(feature = "uniffi"))]
					fn #observe_method_name(
						&self,
						#( #arg_names: #arg_types_owned ),*
					) -> std::pin::Pin<Box<dyn futures::Stream<Item = moka_query::QueryState<#inner_ok_type>> + Send>> {
						#( let _ = &#arg_names; )*
						unimplemented!("observe methods are only implemented on the Cached proxy struct")
					}
				});

				trait_observe_defs.push(quote! {
					#[cfg(feature = "uniffi")]
					fn #observe_method_name(
						&self,
						#( #arg_names: #arg_types_owned ),*
					) -> std::sync::Arc<#stream_struct_name> {
						#( let _ = &#arg_names; )*
						unimplemented!("observe methods are only implemented on the Cached proxy struct")
					}
				});

				trait_impl_methods.push(quote! {
					#[cfg(not(feature = "uniffi"))]
					fn #observe_method_name(
						&self,
						#( #arg_names: #arg_types_owned ),*
					) -> std::pin::Pin<Box<dyn futures::Stream<Item = moka_query::QueryState<#inner_ok_type>> + Send>> {
						let key = format!(#full_key);
						let inner = self.inner.clone();

						let stream = self.global_client.clone().observe(key, move || {
							let inner = inner.clone();
							#( let #arg_names = #arg_names.clone(); )*
							async move {
								#( #captures )*
								inner.#method_name(#(#arg_names),*).await
							}
						});

						Box::pin(stream)
					}

					#[cfg(feature = "uniffi")]
					fn #observe_method_name(
						&self,
						#( #arg_names: #arg_types_owned ),*
					) -> std::sync::Arc<#stream_struct_name> {
						let key = format!(#full_key);
						let inner = self.inner.clone();

						let stream = self.global_client.clone().observe(key, move || {
							let inner = inner.clone();
							#( let #arg_names = #arg_names.clone(); )*
							async move {
								#( #captures )*
								inner.#method_name(#(#arg_names),*).await
							}
						});

						std::sync::Arc::new(#stream_struct_name {
							inner: tokio::sync::Mutex::new(Box::pin(stream))
						})
					}
				});

				ffi_global_functions.push(quote! {
					#[cfg(feature = "uniffi")]
					#[uniffi::export]
					pub fn #ffi_func_name(provider: std::sync::Arc<dyn #trait_name>, #( #arg_names: #arg_types_owned ),*) -> std::sync::Arc<#stream_struct_name> {
						provider.#observe_method_name(#( #arg_names ),*)
					}
				});

				generated_types.push(quote! {
					#[cfg(feature = "uniffi")]
					#[derive(uniffi::Enum)]
					#[derive(Debug, Clone, PartialEq)]
					pub enum #state_enum_name {
						Loading,
						Data { data: #inner_ok_type },
						Error { message: String },
					}

					#[cfg(feature = "uniffi")]
					#[derive(uniffi::Object)]
					pub struct #stream_struct_name {
						inner: tokio::sync::Mutex<std::pin::Pin<Box<dyn futures::Stream<Item = moka_query::QueryState<#inner_ok_type>> + Send>>>,
					}

					#[cfg(feature = "uniffi")]
					#[uniffi::export]
					impl #stream_struct_name {
						pub async fn next(&self) -> Option<#state_enum_name> {
							use futures::StreamExt;
							let mut stream = self.inner.lock().await;
							match stream.next().await {
								Some(moka_query::QueryState::Loading) => Some(#state_enum_name::Loading),
								Some(moka_query::QueryState::Data(d)) => Some(#state_enum_name::Data { data: d }),
								Some(moka_query::QueryState::Error(e)) => Some(#state_enum_name::Error { message: e }),
								None => None,
							}
						}
					}
				});
			}
		}
	}

	for def in trait_observe_defs {
		input_trait.items.push(syn::parse2(def).expect("Failed to parse generated observe definition"));
	}

	let expanded = quote! {
		#( #generated_types )*

		#[async_trait::async_trait]
		#input_trait

		#vis struct #proxy_name<T: #trait_name + 'static> {
			inner: std::sync::Arc<T>,
			global_client: std::sync::Arc<moka_query::GlobalQueryClient>,
		}

		impl<T: #trait_name + 'static> #proxy_name<T> {
			pub fn new(inner: std::sync::Arc<T>, global_client: std::sync::Arc<moka_query::GlobalQueryClient>) -> Self {
				Self { inner, global_client }
			}
		}

		#[async_trait::async_trait]
		impl<T: #trait_name + Send + Sync + 'static> #trait_name for #proxy_name<T> {
			#( #trait_impl_methods )*
		}

		#( #ffi_global_functions )*
	};

	TokenStream::from(expanded)
}
