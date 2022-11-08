// Copyright 2018-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod event;

pub use event::EventMetadata;

use crate::GenerateCode;
use ::core::iter;
use derive_more::From;
use ir::{
    Callable as _,
    HexLiteral,
    IsDocAttribute,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
    ToTokens,
};
use syn::spanned::Spanned as _;

/// Generates code to generate the metadata of the contract.
#[derive(From)]
pub struct Metadata<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(Metadata);

impl GenerateCode for Metadata<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let constructors = self.generate_constructors();
        let messages = self.generate_messages();
        let docs = self
            .contract
            .module()
            .attrs()
            .iter()
            .filter_map(|attr| attr.extract_docs());

        let layout = self.generate_layout();
        let storage_ident = self.contract.module().storage().ident();

        quote! {
            #[cfg(feature = "std")]
            #[cfg(not(feature = "ink-as-dependency"))]
            const _: () = {
                impl ::ink::metadata::ConstructorReturnSpec for #storage_ident {}

                #[no_mangle]
                pub fn __ink_generate_metadata(
                    events: ::ink::prelude::vec::Vec<::ink::metadata::EventSpec>
                ) -> ::ink::metadata::InkProject  {
                    let layout = #layout;
                    ::ink::metadata::layout::ValidateLayout::validate(&layout).unwrap_or_else(|error| {
                        ::core::panic!("metadata ink! generation failed: {}", error)
                    });
                    let contract =
                        ::ink::metadata::ContractSpec::new()
                            .constructors([
                                #( #constructors ),*
                            ])
                            .messages([
                                #( #messages ),*
                            ])
                            .events(
                                events
                            )
                            .docs([
                                #( #docs ),*
                            ])
                            .done();
                    ::ink::metadata::InkProject::new(layout, contract)
                }
            };
        }
    }
}

impl Metadata<'_> {
    fn generate_layout(&self) -> TokenStream2 {
        let storage_span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let key = quote! { <#storage_ident as ::ink::storage::traits::StorageKey>::KEY };

        let layout_key = quote! {
            <::ink::metadata::layout::LayoutKey
                as ::core::convert::From<::ink::primitives::Key>>::from(#key)
        };
        quote_spanned!(storage_span=>
            // Wrap the layout of the contract into the `RootLayout`, because
            // contract storage key is reserved for all packed fields
            ::ink::metadata::layout::Layout::Root(::ink::metadata::layout::RootLayout::new(
                #layout_key,
                <#storage_ident as ::ink::storage::traits::StorageLayout>::layout(
                    &#key,
                ),
            ))
        )
    }

    /// Generates ink! metadata for all ink! smart contract constructors.
    #[allow(clippy::redundant_closure)] // We are getting arcane lifetime errors otherwise.
    fn generate_constructors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .map(|constructor| Self::generate_constructor(constructor))
    }

    /// Generates ink! metadata for a single ink! constructor.
    fn generate_constructor(
        constructor: ir::CallableWithSelector<ir::Constructor>,
    ) -> TokenStream2 {
        let span = constructor.span();
        let docs = constructor
            .attrs()
            .iter()
            .filter_map(|attr| attr.extract_docs());
        let selector_bytes = constructor.composed_selector().hex_lits();
        let is_payable = constructor.is_payable();
        let constructor = constructor.callable();
        let ident = constructor.ident();
        let args = constructor.inputs().map(Self::generate_dispatch_argument);
        let ret_ty = Self::generate_constructor_return_type(constructor.output());
        quote_spanned!(span=>
            ::ink::metadata::ConstructorSpec::from_label(::core::stringify!(#ident))
                .selector([
                    #( #selector_bytes ),*
                ])
                .args([
                    #( #args ),*
                ])
                .payable(#is_payable)
                .returns(#ret_ty)
                .docs([
                    #( #docs ),*
                ])
                .done()
        )
    }

    /// Generates the ink! metadata for the given parameter and parameter type.
    fn generate_dispatch_argument(pat_type: &syn::PatType) -> TokenStream2 {
        let ident = match &*pat_type.pat {
            syn::Pat::Ident(ident) => &ident.ident,
            _ => unreachable!("encountered ink! dispatch input with missing identifier"),
        };
        let type_spec = generate_type_spec(&pat_type.ty);
        quote! {
            ::ink::metadata::MessageParamSpec::new(::core::stringify!(#ident))
                .of_type(#type_spec)
                .done()
        }
    }

    /// Generates the ink! metadata segments iterator for the given type of a constructor.
    fn generate_constructor_type_segments(ty: &syn::Type) -> TokenStream2 {
        fn without_display_name() -> TokenStream2 {
            quote! { None }
        }
        if let syn::Type::Path(type_path) = ty {
            if type_path.qself.is_some() {
                return without_display_name()
            }
            let path = &type_path.path;
            if path.segments.is_empty() {
                return without_display_name()
            }
            let segs = path
                .segments
                .iter()
                .map(|seg| &seg.ident)
                .collect::<Vec<_>>();
            quote! {
                Some(::core::iter::IntoIterator::into_iter([ #( ::core::stringify!(#segs) ),* ])
                        .map(::core::convert::AsRef::as_ref)
                )
            }
        } else {
            without_display_name()
        }
    }

    /// Generates the ink! metadata for all ink! smart contract messages.
    fn generate_messages(&self) -> Vec<TokenStream2> {
        let mut messages = Vec::new();
        let inherent_messages = self.generate_inherent_messages();
        let trait_messages = self.generate_trait_messages();
        messages.extend(inherent_messages);
        messages.extend(trait_messages);
        messages
    }

    /// Generates the ink! metadata for all inherent ink! smart contract messages.
    fn generate_inherent_messages(&self) -> Vec<TokenStream2> {
        self.contract
            .module()
            .impls()
            .filter(|item_impl| item_impl.trait_path().is_none())
            .flat_map(|item_impl| item_impl.iter_messages())
            .map(|message| {
                let span = message.span();
                let docs = message
                    .attrs()
                    .iter()
                    .filter_map(|attr| attr.extract_docs());
                let selector_bytes = message.composed_selector().hex_lits();
                let is_payable = message.is_payable();
                let message = message.callable();
                let mutates = message.receiver().is_ref_mut();
                let ident = message.ident();
                let args = message.inputs().map(Self::generate_dispatch_argument);
                let ret_ty = Self::generate_return_type(message.output());
                quote_spanned!(span =>
                    ::ink::metadata::MessageSpec::from_label(::core::stringify!(#ident))
                        .selector([
                            #( #selector_bytes ),*
                        ])
                        .args([
                            #( #args ),*
                        ])
                        .returns(#ret_ty)
                        .mutates(#mutates)
                        .payable(#is_payable)
                        .docs([
                            #( #docs ),*
                        ])
                        .done()
                )
            })
            .collect()
    }

    /// Generates the ink! metadata for all inherent ink! smart contract messages.
    fn generate_trait_messages(&self) -> Vec<TokenStream2> {
        let storage_ident = self.contract.module().storage().ident();
        self.contract
            .module()
            .impls()
            .filter_map(|item_impl| {
                item_impl
                    .trait_path()
                    .map(|trait_path| {
                        let trait_ident = item_impl.trait_ident().expect(
                            "must have an ink! trait identifier if it is an ink! trait implementation"
                        );
                        iter::repeat((trait_ident, trait_path)).zip(item_impl.iter_messages())
                    })
            })
            .flatten()
            .map(|((trait_ident, trait_path), message)| {
                let message_span = message.span();
                let message_ident = message.ident();
                let message_docs = message
                    .attrs()
                    .iter()
                    .filter_map(|attr| attr.extract_docs());
                let message_args = message
                    .inputs()
                    .map(Self::generate_dispatch_argument);
                let mutates = message.receiver().is_ref_mut();
                let local_id = message.local_id().hex_padded_suffixed();
                let is_payable = quote! {{
                    <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::reflect::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink::reflect::TraitMessageInfo<#local_id>>::PAYABLE
                }};
                let selector = quote! {{
                    <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::reflect::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink::reflect::TraitMessageInfo<#local_id>>::SELECTOR
                }};
                let ret_ty = Self::generate_return_type(message.output());
                let label = [trait_ident.to_string(), message_ident.to_string()].join("::");
                quote_spanned!(message_span=>
                    ::ink::metadata::MessageSpec::from_label(#label)
                        .selector(#selector)
                        .args([
                            #( #message_args ),*
                        ])
                        .returns(#ret_ty)
                        .mutates(#mutates)
                        .payable(#is_payable)
                        .docs([
                            #( #message_docs ),*
                        ])
                        .done()
                )
            })
            .collect()
    }

    /// Generates ink! metadata for the given return type.
    fn generate_return_type(ret_ty: Option<&syn::Type>) -> TokenStream2 {
        match ret_ty {
            None => {
                quote! {
                    ::ink::metadata::ReturnTypeSpec::new(::core::option::Option::None)
                }
            }
            Some(ty) => {
                let type_spec = generate_type_spec(ty);
                quote! {
                    ::ink::metadata::ReturnTypeSpec::new(#type_spec)
                }
            }
        }
    }

    /// Generates ink! metadata for the given return type of a constructor.
    /// If the constructor return type is not `Result`,
    /// the metadata will not display any type spec for the return type.
    /// Otherwise, the return type spec is `Result<(), E>`.
    fn generate_constructor_return_type(ret_ty: Option<&syn::Type>) -> TokenStream2 {
        match ret_ty {
            None => {
                quote! {
                    ::ink::metadata::ReturnTypeSpec::new(::core::option::Option::None)
                }
            }
            Some(syn::Type::Path(syn::TypePath { qself: None, path }))
            if path.is_ident("Self") =>
                {
                    quote! { ::ink::metadata::ReturnTypeSpec::new(::core::option::Option::None)}
                }
            Some(ty) => {
                let type_token = Self::replace_self_with_unit(ty);
                let segments = Self::generate_constructor_type_segments(ty);
                quote! {
                    ::ink::metadata::ReturnTypeSpec::new(
                        <#type_token as ::ink::metadata::ConstructorReturnSpec>::generate(#segments)
                    )
                }
            }
        }
    }

    /// Helper function to replace all occurrences of `Self` with `()`.
    fn replace_self_with_unit(ty: &syn::Type) -> TokenStream2 {
        if ty.to_token_stream().to_string().contains("< Self") {
            let s = ty.to_token_stream().to_string().replace("< Self", "< ()");
            s.parse().unwrap()
        } else {
            ty.to_token_stream()
        }
    }
}

/// Generates the ink! metadata for the given type.
fn generate_type_spec(ty: &syn::Type) -> TokenStream2 {
    fn without_display_name(ty: &syn::Type) -> TokenStream2 {
        quote! { ::ink::metadata::TypeSpec::of_type::<#ty>() }
    }
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_some() {
            return without_display_name(ty)
        }
        let path = &type_path.path;
        if path.segments.is_empty() {
            return without_display_name(ty)
        }
        let segs = path
            .segments
            .iter()
            .map(|seg| &seg.ident)
            .collect::<Vec<_>>();
        quote! {
                ::ink::metadata::TypeSpec::with_name_segs::<#ty, _>(
                    ::core::iter::IntoIterator::into_iter([ #( ::core::stringify!(#segs) ),* ])
                        .map(::core::convert::AsRef::as_ref)
                )
            }
    } else {
        without_display_name(ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Extracts and collects the contents of the Rust documentation attributes.
    fn extract_doc_attributes(attrs: &[syn::Attribute]) -> Vec<String> {
        attrs
            .iter()
            .filter_map(|attr| attr.extract_docs())
            .collect()
    }

    #[test]
    fn extract_doc_comments_works() {
        assert_eq!(
            extract_doc_attributes(&[syn::parse_quote!( #[doc = r"content"] )]),
            vec!["content".to_string()],
        );
        assert_eq!(
            extract_doc_attributes(&[syn::parse_quote!(
                /// content
            )]),
            vec![" content".to_string()],
        );
        assert_eq!(
            extract_doc_attributes(&[syn::parse_quote!(
                /**
                 * Multi-line comments
                 * may span many,
                 * many lines
                 */
            )]),
            vec![r"
                 * Multi-line comments
                 * may span many,
                 * many lines
                 "
            .to_string()],
        );
        assert_eq!(
            extract_doc_attributes(&[
                syn::parse_quote!(
                    /// multiple
                ),
                syn::parse_quote!(
                    /// single
                ),
                syn::parse_quote!(
                    /// line
                ),
                syn::parse_quote!(
                    /// comments
                ),
            ]),
            vec![
                " multiple".to_string(),
                " single".to_string(),
                " line".to_string(),
                " comments".to_string(),
            ],
        );
        assert_eq!(
            extract_doc_attributes(&[
                syn::parse_quote!( #[doc = r"a"] ),
                syn::parse_quote!( #[non_doc] ),
                syn::parse_quote!( #[doc = r"b"] ),
                syn::parse_quote!( #[derive(NonDoc)] ),
                syn::parse_quote!( #[doc = r"c"] ),
                syn::parse_quote!( #[docker = false] ),
                syn::parse_quote!( #[doc = r"d"] ),
                syn::parse_quote!( #[doc(Nope)] ),
                syn::parse_quote!( #[doc = r"e"] ),
            ]),
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
            ],
        )
    }

    #[test]
    fn constructor_return_type_works() {
        let expected_no_ret_type_spec = ":: ink :: metadata :: ReturnTypeSpec :: new (:: core :: option :: Option :: None)";

        let actual = Metadata::generate_constructor_return_type(None);
        assert_eq!(&actual.to_string(), expected_no_ret_type_spec);

        match syn::parse_quote!( -> Self ) {
            syn::ReturnType::Type(_, t) => {
                let actual = Metadata::generate_constructor_return_type(Some(&t));
                assert_eq!(&actual.to_string(), expected_no_ret_type_spec);
            }
            _ => unreachable!(),
        }

        match syn::parse_quote!( -> Result<Self, ()> ) {
            syn::ReturnType::Type(_, t) => {
                let actual = Metadata::generate_constructor_return_type(Some(&t));
                let expected = ":: ink :: metadata :: ReturnTypeSpec :: new (< Result < () , () > as :: ink :: metadata :: ConstructorReturnSpec > :: generate (Some (:: core :: iter :: IntoIterator :: into_iter ([:: core :: stringify ! (Result)]) . map (:: core :: convert :: AsRef :: as_ref))))";
                assert_eq!(&actual.to_string(), expected);
            }
            _ => unreachable!(),
        }
    }
}
