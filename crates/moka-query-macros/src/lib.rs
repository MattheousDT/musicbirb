extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
	parse_macro_input, Expr, ExprArray, ExprLit, FnArg, ItemTrait, Lit, LitStr, Meta, ReturnType, TraitItem, Type,
};

fn strip_reference(ty: &Type) -> Type {
	if let Type::Reference(type_ref) = ty {
		let inner = &*type_ref.elem;
		// Special case: &str must become String for UniFFI and ownership
		if let Type::Path(p) = inner {
			if p.path.is_ident("str") {
				return syn::parse_quote!(String);
			}
		}
		*type_ref.elem.clone()
	} else {
		ty.clone()
	}
}

fn is_unit_type(ty: &Type) -> bool {
	if let Type::Tuple(tuple) = ty {
		tuple.elems.is_empty()
	} else {
		false
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
	let input_trait = parse_macro_input!(item as ItemTrait);

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
	let mut uniffi_inherent_methods = Vec::new();

	for item in &input_trait.items {
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
			let mut retries: u32 = 0;

			for attr in &method.attrs {
				if attr.path().is_ident("query") {
					is_query = true;
					if let Meta::List(meta) = &attr.meta {
						let nested = meta
							.parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
							.unwrap();
						for m in nested {
							if let Meta::NameValue(nv) = m {
								if nv.path.is_ident("key") {
									if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = nv.value {
										query_key_template = s.value();
									}
								} else if nv.path.is_ident("retries") {
									if let Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) = nv.value {
										retries = i.base10_parse().unwrap_or(0);
									}
								}
							}
						}
					}
				} else if attr.path().is_ident("mutation") {
					is_mutation = true;
					if let Meta::List(meta) = &attr.meta {
						let expr: syn::ExprAssign = syn::parse2(meta.tokens.clone()).unwrap();
						if let Expr::Array(ExprArray { elems, .. }) = *expr.right {
							for elem in elems {
								if let Expr::Lit(ExprLit {
									lit: Lit::Str(lit_str), ..
								}) = elem
								{
									invalidates_patterns.push(lit_str.value());
								}
							}
						}
					}
				}
			}

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

			// 1. STANDARD TRAIT IMPLS (For Rust internals)
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

			// 2. UNIFFI INHERENT EXPORTS (Bypasses all Trait limitations in Swift!)
			if is_mutation {
				uniffi_inherent_methods.push(quote! {
					pub #sig {
						let res = self.inner.#method_name(#(#arg_names),*).await;
						if res.is_ok() {
							#( self.global_client.invalidate_pattern(#invalidates_patterns).await; )*
						}
						res
					}
				});

				let method_camel = method_name
					.to_string()
					.split('_')
					.map(|s| format!("{}{}", &s[..1].to_uppercase(), &s[1..]))
					.collect::<String>();
				let stream_struct_name = format_ident!("Mutate{}Stream", method_camel);
				let state_enum_name = format_ident!("Mutate{}State", method_camel);
				let execute_method_name = format_ident!("execute_{}", method_name);

				let inner_ok_type_opt = extract_ok_type(output);
				let mut ok_type_for_enum = quote! { () };
				let mut is_unit = true;

				if let Some(ty) = &inner_ok_type_opt {
					if !is_unit_type(ty) {
						ok_type_for_enum = quote! { #ty };
						is_unit = false;
					}
				}

				let data_variant = if is_unit {
					quote! { Data }
				} else {
					quote! { Data { data: #ok_type_for_enum } }
				};

				let data_yield = if is_unit {
					quote! {
						let _ = data; // Suppress unused warnings
						yield #state_enum_name::Data;
					}
				} else {
					quote! { yield #state_enum_name::Data { data }; }
				};

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

				uniffi_inherent_methods.push(quote! {
					pub fn #execute_method_name(&self, #( #arg_names: #arg_types_owned ),*) -> std::sync::Arc<#stream_struct_name> {
						let inner = self.inner.clone();
						let global_client = self.global_client.clone();
						#( let #arg_names = #arg_names.clone(); )*

						let stream = moka_query::async_stream::stream! {
							yield #state_enum_name::Loading;
							#( #captures )*
							let res = inner.#method_name(#(#arg_names),*).await;
							match res {
								Ok(data) => {
									#( global_client.invalidate_pattern(#invalidates_patterns).await; )*
									#data_yield
								}
								Err(err) => yield #state_enum_name::Error { message: err.to_string() }
							}
						};

						std::sync::Arc::new(#stream_struct_name {
							inner: tokio::sync::Mutex::new(Box::pin(stream)),
						})
					}
				});

				generated_types.push(quote! {
					#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
					#[derive(Debug, Clone, PartialEq)]
					pub enum #state_enum_name {
						Loading, #data_variant, Error { message: String },
					}

					#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
					pub struct #stream_struct_name {
						inner: tokio::sync::Mutex<std::pin::Pin<Box<dyn futures::Stream<Item = #state_enum_name> + Send>>>,
					}

					#[cfg_attr(feature = "uniffi", uniffi::export)]
					impl #stream_struct_name {
						pub async fn next(&self) -> Option<#state_enum_name> {
							use futures::StreamExt;
							let mut stream = self.inner.lock().await;
							stream.next().await
						}
					}
				});
			} else if is_async {
				uniffi_inherent_methods.push(quote! {
					pub #sig { self.inner.#method_name(#(#arg_names),*).await }
				});
			} else {
				uniffi_inherent_methods.push(quote! {
					pub #sig { self.inner.#method_name(#(#arg_names),*) }
				});
			}

			// 3. GENERATE THE OBSERVE METHODS natively on the struct
			if is_query {
				let inner_ok_type = extract_ok_type(output).unwrap();
				let observe_method_name = format_ident!("observe_{}", method_name);
				let full_key = format!("{}/{}", namespace, query_key_template);
				let method_camel = method_name
					.to_string()
					.split('_')
					.map(|s| format!("{}{}", &s[..1].to_uppercase(), &s[1..]))
					.collect::<String>();
				let stream_struct_name = format_ident!("Observe{}Stream", method_camel);
				let state_enum_name = format_ident!("Observe{}State", method_camel);

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

				let set_method_name = format_ident!("set_cached_{}", method_name);

				uniffi_inherent_methods.push(quote! {
					pub async fn #set_method_name(&self, #( #arg_names: #arg_types_owned, )* data: #inner_ok_type) {
						let key = format!(#full_key);
						self.global_client.set_query_data(&key, data).await;
					}

					pub fn #observe_method_name(&self, #( #arg_names: #arg_types_owned ),*) -> std::sync::Arc<#stream_struct_name> {
						let key = format!(#full_key);
						let inner = self.inner.clone();
						let stream = self.global_client.clone().observe(key.clone(), #retries, move || {
							let inner = inner.clone();
							#( let #arg_names = #arg_names.clone(); )*
							async move { #( #captures )* inner.#method_name(#(#arg_names),*).await }
						});
						std::sync::Arc::new(#stream_struct_name {
							inner: tokio::sync::Mutex::new(Box::pin(stream)),
							global_client: self.global_client.clone(),
							key,
						})
					}
				});

				generated_types.push(quote! {
					#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
					#[derive(Debug, Clone, PartialEq)]
					pub enum #state_enum_name {
						Loading, Data { data: #inner_ok_type }, Error { message: String },
					}

					#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
					pub struct #stream_struct_name {
						inner: tokio::sync::Mutex<std::pin::Pin<Box<dyn futures::Stream<Item = moka_query::QueryState<#inner_ok_type>> + Send>>>,
						global_client: std::sync::Arc<moka_query::GlobalQueryClient>,
						key: String,
					}

					#[cfg_attr(feature = "uniffi", uniffi::export)]
					impl #stream_struct_name {
						pub fn current_cached_state(&self) -> Option<#state_enum_name> {
							match self.global_client.get_cached_state::<#inner_ok_type>(&self.key) {
								Some(moka_query::QueryState::Loading) => Some(#state_enum_name::Loading),
								Some(moka_query::QueryState::Data(d)) => Some(#state_enum_name::Data { data: d }),
								Some(moka_query::QueryState::Error(e)) => Some(#state_enum_name::Error { message: e }),
								None => None,
							}
						}

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

	let mut cleaned_trait = input_trait.clone();
	for item in &mut cleaned_trait.items {
		if let TraitItem::Fn(method) = item {
			method
				.attrs
				.retain(|attr| !attr.path().is_ident("query") && !attr.path().is_ident("mutation"));
		}
	}

	let expanded = quote! {
		#( #generated_types )*

		#[async_trait::async_trait]
		#cleaned_trait

		#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
		#vis struct #proxy_name {
			inner: std::sync::Arc<dyn #trait_name + Send + Sync>,
			global_client: std::sync::Arc<moka_query::GlobalQueryClient>,
		}

		impl #proxy_name {
			pub fn new(inner: std::sync::Arc<dyn #trait_name + Send + Sync>, global_client: std::sync::Arc<moka_query::GlobalQueryClient>) -> Self {
				Self { inner, global_client }
			}
		}

		#[cfg_attr(feature = "uniffi", uniffi::export)]
		impl #proxy_name {
			#( #uniffi_inherent_methods )*

			// Native Manual Invalidation!
			pub async fn moka_invalidate(&self, pattern: String) {
				self.global_client.invalidate_pattern(&pattern).await;
			}
		}

		#[async_trait::async_trait]
		impl #trait_name for #proxy_name {
			#( #trait_impl_methods )*
		}
	};

	TokenStream::from(expanded)
}
