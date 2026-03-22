extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Block, ImplItem, Item, parse_macro_input, parse_quote};

/// Rewrites all `async fn` blocks inside an `impl` block to be transparently
/// wrapped in `crate::ffi::WithTokio`.
/// Automatically applies `#[async_trait::async_trait]` to Traits and Trait Impls.
#[proc_macro_attribute]
pub fn async_ffi(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let parsed_item = parse_macro_input!(item as Item);

	match parsed_item {
		Item::Impl(mut input_impl) => {
			for impl_item in &mut input_impl.items {
				if let ImplItem::Fn(method) = impl_item {
					// We only wrap methods that are explicitly marked as async
					if method.sig.asyncness.is_some() {
						let original_block = &method.block;

						let new_block: Block = parse_quote! {
							{
								#[cfg(feature = "ffi")]
								let __musicbirb_res = crate::ffi::WithTokio::new(async move { #original_block }).await;

								#[cfg(not(feature = "ffi"))]
								let __musicbirb_res = { #original_block };

								__musicbirb_res
							}
						};

						method.block = new_block;
					}
				}
			}

			// If this is a trait implementation block, automatically append #[async_trait]
			if input_impl.trait_.is_some() {
				TokenStream::from(quote! {
					#[::async_trait::async_trait]
					#input_impl
				})
			} else {
				// Otherwise (e.g., impl Musicbirb), leave it as a normal block
				TokenStream::from(quote! { #input_impl })
			}
		}
		Item::Trait(input_trait) => {
			// If applied directly to the trait definition, simply forward it with #[async_trait]
			TokenStream::from(quote! {
				#[::async_trait::async_trait]
				#input_trait
			})
		}
		_ => TokenStream::from(quote! {
			compile_error!("wrap_ffi_async can only be applied to impl blocks or trait definitions");
		}),
	}
}
