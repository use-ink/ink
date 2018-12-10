use crate::{
	contract::{
		ContractModule,
		ContractStruct,
		ContractEnum,
		ContractItem,
	},
	utils::{
		has_attribute,
	},
	error::{
		Error,
	}
};

use proc_macro2;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream, Result}
};
use itertools::{Either, Itertools};

impl Parse for ContractModule {
	fn parse(input: ParseStream) -> Result<Self> {
		let item_mod: syn::ItemMod = input.parse()?;
		let (brace, items) = match item_mod.content {
			Some(items) => items,
			None => return Err(input.error(
				Error::mod_without_body(item_mod.ident)
			))
		};
		let vis = match item_mod.vis {
			syn::Visibility::Public(vis) => vis,
			invalid_vis => return Err(input.error(
				Error::invalid_mod_visibility(item_mod.ident, invalid_vis)
			))
		};

		let (items, contracts): (Vec<_>, Vec<_>) = items.into_iter().partition_map(|item| {
			match item {
				syn::Item::Struct(item_struct) => {
					if has_attribute(&item_struct.attrs, "pdsl::contract") {
						Either::Right(
							ContractItem::Struct(
								ContractStruct{ item: item_struct }
							)
						)
					} else {
						Either::Left(syn::Item::Struct(item_struct))
					}
				}
				syn::Item::Enum(item_enum) => {
					if has_attribute(&item_enum.attrs, "pdsl::contract") {
						Either::Right(
							ContractItem::Enum(
								ContractEnum{ item: item_enum }
							)
						)
					} else {
						Either::Left(syn::Item::Enum(item_enum))
					}
				}
				item => Either::Left(item)
			}
		});

		for contract in &contracts {
			match contract {
				ContractItem::Struct(item_struct) => {
					println!("[psdl::contract] found struct contract => {:?}", item_struct.item.ident);
				}
				ContractItem::Enum(item_enum) => {
					println!("[psdl::contract] found enum contract => {:?}", item_enum.item.ident);
				}
			}
		}

		Ok(ContractModule{
			vis,
			mod_tok: item_mod.mod_token,
			ident: item_mod.ident,
			attrs: item_mod.attrs,
			brace,
			items,
			contracts,
		})
	}
}

impl quote::ToTokens for ContractStruct {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		println!("[pdsl_derive] ContractStruct::to_tokens");
		match &self.item.fields {
			syn::Fields::Named(fields_named) => {
				// Generate field code
				for field in fields_named.named.iter() {
					let name = field.ident.clone().unwrap();
					let marker_name = syn::Ident::new(
						&format!(
							"FieldMarker_{}",
							name.to_string()
						),
						proc_macro2::Span::call_site()
					);
					let marker_storage_type = &field.ty;
					let marker_name_str = syn::LitStr::new(
						&name.to_string(),
						proc_macro2::Span::call_site()
					);
					let field_toks: proc_macro2::TokenStream = syn::parse_quote! {
						#[derive(Debug, Copy, Clone)]
						pub struct #marker_name;

						impl pdsl_core::Storage for #marker_name {
							type Type = #marker_storage_type;
							const KEY: pdsl_core::Key = pdsl_core::Key([0x42; 32]);
						}

						impl pdsl_core::Name for #marker_name {
							const NAME: &'static str = #marker_name_str ;
						}

					};
					tokens.extend(field_toks);

					println!("[pdsl_derive] ContractStruct:to_tokens -> gen_field({})", name);
				}
				let vis_toks = &self.item.vis;
				let struct_toks = self.item.struct_token;
				let ident_toks = &self.item.ident;
				let semi_toks = self.item.semi_token;
				for attr in self.item.attrs.iter() {
					let attr_toks: proc_macro2::TokenStream = syn::parse_quote! { #attr };
					tokens.extend(attr_toks);
				}
				let struct_toks: proc_macro2::TokenStream = syn::parse_quote! {
					// #attrs_toks
					#vis_toks #struct_toks #ident_toks
				};
				tokens.extend(struct_toks);
				fields_named.brace_token.surround(tokens, |tokens| {
					for field in fields_named.named.iter() {
						let name = field.ident.clone().unwrap();
						let marker_name = syn::Ident::new(
							&format!(
								"FieldMarker_{}",
								name.to_string()
							),
							proc_macro2::Span::call_site()
						);
						let struct_toks: proc_macro2::TokenStream = syn::parse_quote! {
							#name: pdsl_core::Field< #marker_name >,
						};
						tokens.extend(struct_toks);
					}
				});
				let opt_semi_toks: proc_macro2::TokenStream = syn::parse_quote! {
					#semi_toks
				};
				tokens.extend(opt_semi_toks);
			}
			syn::Fields::Unnamed(fields_unnamed) => {
				// ERROR: no yet handled (maybe in future)
			}
			syn::Fields::Unit => {
				// ERROR: unreachable
			}
		}
	}
}

impl quote::ToTokens for ContractEnum {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		unimplemented!()
	}
}

impl quote::ToTokens for ContractItem {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			ContractItem::Struct(item_struct) => item_struct.to_tokens(tokens),
			ContractItem::Enum(item_enum) => item_enum.to_tokens(tokens),
		}
	}
}

impl quote::ToTokens for ContractModule {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		{
			let header: proc_macro2::TokenStream = syn::parse_quote!{
				use core::intrinsics;

				use parity_codec_derive::{Encode, Decode};
				use parity_codec::{Encode, Decode};

				// Use `wee_alloc` as the global allocator.
				use wee_alloc;

				#[global_allocator]
				static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

				// #[panic_handler]
				// #[no_mangle]
				// pub fn panic(_info: &::core::panic::PanicInfo) -> ! {
				// 	unsafe {
				// 		intrinsics::abort()
				// 	}
				// }

				// #[alloc_error_handler]
				// pub extern fn oom(_: ::core::alloc::Layout) -> ! {
				// 	unsafe {
				// 		intrinsics::abort();
				// 	}
				// }

				// We need to define `deploy` function,
				// so the wasm-build will produce a constructor binary.
				#[no_mangle]
				pub extern "C" fn deploy() {}
			};
			tokens.extend(header);
		}

		/// Returns the tokens of the external definitions module.
		pub fn ext_tokens() -> proc_macro2::TokenStream {
			syn::parse_quote!{
				use pdsl_core;
			}
		}
		tokens.extend( ext_tokens() );

		for outer_attr in self
			.attrs
			.iter()
			.filter(|attr| {
				if let syn::AttrStyle::Outer = attr.style { true } else { false }
			})
		{
			outer_attr.to_tokens(tokens)
		}
		self.vis.to_tokens(tokens);
		self.mod_tok.to_tokens(tokens);
		self.ident.to_tokens(tokens);
		self.brace.surround(tokens, |tokens| {
			{
				let use_ext: proc_macro2::TokenStream = syn::parse_quote!{ use pdsl_core::ext; };
				tokens.extend(use_ext);
			}
			for inner_attr in self
				.attrs
				.iter()
				.filter(|attr| {
					if let syn::AttrStyle::Inner(_vis) = attr.style { true } else { false }
				})
			{
				inner_attr.to_tokens(tokens)
			}
			for item in &self.items {
				item.to_tokens(tokens)
			}
			for contract in &self.contracts {
				contract.to_tokens(tokens)
			}
		});
	}
}