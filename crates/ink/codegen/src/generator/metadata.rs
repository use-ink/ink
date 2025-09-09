// Copyright (C) Use Ink (UK) Ltd.
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

use crate::GenerateCode;
use ::core::iter;
use derive_more::From;
use ir::{
    Callable as _,
    HexLiteral,
    IsDocAttribute,
};
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    parse_quote,
    spanned::Spanned as _,
};

/// Generates code to generate the metadata of the contract.
#[derive(From)]
pub struct Metadata<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(Metadata);

impl GenerateCode for Metadata<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let contract = self.generate_contract();
        let layout = self.generate_layout();

        quote! {
            #[cfg(feature = "std")]
            #[cfg(not(feature = "ink-as-dependency"))]
            #[cfg(not(ink_abi = "sol"))]
            const _: () = {
                #[unsafe(no_mangle)]
                pub fn __ink_generate_metadata() -> ::ink::metadata::InkProject  {
                    let layout = #layout;
                    ::ink::metadata::layout::ValidateLayout::validate(&layout).unwrap_or_else(|error| {
                        ::core::panic!("metadata ink! generation failed: {}", error)
                    });
                    ::ink::metadata::InkProject::new(layout, #contract)
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
                ::ink::scale_info::meta_type::<#storage_ident>(),
            ))
        )
    }

    fn generate_contract(&self) -> TokenStream2 {
        let constructors = self.generate_constructors();
        let messages = self.generate_messages();
        let docs = self
            .contract
            .module()
            .attrs()
            .iter()
            .filter_map(|attr| attr.extract_docs());
        let error_ty = syn::parse_quote! {
            ::ink::LangError
        };
        let error = generate_type_spec(&error_ty);
        let environment = self.generate_environment();
        quote! {
            ::ink::metadata::ContractSpec::new()
                .constructors([
                    #( #constructors ),*
                ])
                .messages([
                    #( #messages ),*
                ])
                .events(
                    ::ink::collect_events()
                )
                .docs([
                    #( #docs ),*
                ])
                .lang_error(
                     #error
                )
                .environment(
                    #environment
                )
                .done()
        }
    }

    /// Generates ink! metadata for all ink! smart contract constructors.
    #[allow(clippy::redundant_closure)] // We are getting arcane lifetime errors otherwise.
    fn generate_constructors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .map(|constructor| self.generate_constructor(constructor))
    }

    /// Generates ink! metadata for a single ink! constructor.
    fn generate_constructor(
        &self,
        constructor: ir::CallableWithSelector<ir::Constructor>,
    ) -> TokenStream2 {
        let span = constructor.span();
        let docs = constructor
            .attrs()
            .iter()
            .filter_map(|attr| attr.extract_docs());
        let selector_bytes = constructor.composed_selector().hex_lits();
        let selector_id = constructor.composed_selector().into_be_u32();
        let is_payable = constructor.is_payable();
        let is_default = constructor.is_default();
        let constructor = constructor.callable();
        let name = constructor
            .name()
            .map(ToString::to_string)
            .unwrap_or_else(|| constructor.ident().to_string());
        let args = constructor.inputs().map(Self::generate_dispatch_argument);
        let storage_ident = self.contract.module().storage().ident();
        let ret_ty = Self::generate_constructor_return_type(storage_ident, selector_id);
        let cfg_attrs = constructor.get_cfg_attrs(span);
        quote_spanned!(span=>
            #( #cfg_attrs )*
            ::ink::metadata::ConstructorSpec::from_label(#name)
                .selector([
                    #( #selector_bytes ),*
                ])
                .args([
                    #( #args ),*
                ])
                .payable(#is_payable)
                .default(#is_default)
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
                let is_default = message.is_default();
                let message = message.callable();
                let mutates = message.receiver().is_ref_mut();
                let name = message
                    .name()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| message.ident().to_string());
                let args = message.inputs().map(Self::generate_dispatch_argument);
                let cfg_attrs = message.get_cfg_attrs(span);
                let ret_ty =
                    Self::generate_message_return_type(&message.wrapped_output());
                quote_spanned!(span =>
                    #( #cfg_attrs )*
                    ::ink::metadata::MessageSpec::from_label(#name)
                        .selector([
                            #( #selector_bytes ),*
                        ])
                        .args([
                            #( #args ),*
                        ])
                        .returns(#ret_ty)
                        .mutates(#mutates)
                        .payable(#is_payable)
                        .default(#is_default)
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
                let message_name = message
                    .name()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| message.ident().to_string());
                let message_docs = message
                    .attrs()
                    .iter()
                    .filter_map(|attr| attr.extract_docs());
                let message_args = message
                    .inputs()
                    .map(Self::generate_dispatch_argument);
                let cfg_attrs = message.get_cfg_attrs(message_span);
                let mutates = message.receiver().is_ref_mut();
                let local_id = message.local_id().hex_padded_suffixed();
                let is_payable = quote! {{
                    <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::env::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink::reflect::TraitMessageInfo<#local_id>>::PAYABLE
                }};
                let selector = quote! {{
                    <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::env::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink::reflect::TraitMessageInfo<#local_id>>::SELECTOR
                }};
                let ret_ty = Self::generate_message_return_type(&message.wrapped_output());
                let label = [trait_ident.to_string(), message_name].join("::");
                quote_spanned!(message_span=>
                    #( #cfg_attrs )*
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
    fn generate_message_return_type(ret_ty: &syn::Type) -> TokenStream2 {
        let type_spec = generate_type_spec(ret_ty);
        quote! {
            ::ink::metadata::ReturnTypeSpec::new(#type_spec)
        }
    }

    /// Generates ink! metadata for the storage with given selector and ident.
    fn generate_constructor_return_type(
        storage_ident: &Ident,
        selector_id: u32,
    ) -> TokenStream2 {
        let span = storage_ident.span();
        let constructor_info = quote_spanned!(span =>
            < #storage_ident as ::ink::reflect::DispatchableConstructorInfo<#selector_id>>
        );

        quote_spanned!(span=>
            ::ink::metadata::ReturnTypeSpec::new(if #constructor_info::IS_RESULT {
                ::ink::metadata::TypeSpec::with_name_str::<
                    ::ink::ConstructorResult<::core::result::Result<(), #constructor_info::Error>>,
                >("ink_primitives::ConstructorResult")
            } else {
                ::ink::metadata::TypeSpec::with_name_str::<
                    ::ink::ConstructorResult<()>,
                >("ink_primitives::ConstructorResult")
            })
        )
    }

    fn generate_environment(&self) -> TokenStream2 {
        let span = self.contract.module().span();

        let account_id: syn::Type = parse_quote!(AccountId);
        let balance: syn::Type = parse_quote!(Balance);
        let hash: syn::Type = parse_quote!(Hash);
        let timestamp: syn::Type = parse_quote!(Timestamp);
        let block_number: syn::Type = parse_quote!(BlockNumber);

        let account_id = generate_type_spec(&account_id);
        let balance = generate_type_spec(&balance);
        let hash = generate_type_spec(&hash);
        let timestamp = generate_type_spec(&timestamp);
        let block_number = generate_type_spec(&block_number);
        let buffer_size_const = quote!(::ink::env::BUFFER_SIZE);
        quote_spanned!(span=>
            ::ink::metadata::EnvironmentSpec::new()
                .account_id(#account_id)
                .balance(#balance)
                .hash(#hash)
                .timestamp(#timestamp)
                .block_number(#block_number)
                .native_to_eth_ratio(NATIVE_TO_ETH_RATIO)
                .static_buffer_size(#buffer_size_const)
                .done()
        )
    }
}

/// Generates the ink! metadata for the given type.
pub fn generate_type_spec(ty: &syn::Type) -> TokenStream2 {
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
                ::core::iter::Iterator::map(
                    ::core::iter::IntoIterator::into_iter([ #( ::core::stringify!(#segs) ),* ]),
                    ::core::convert::AsRef::as_ref
                )
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
            vec![
                r"
                 * Multi-line comments
                 * may span many,
                 * many lines
                 "
                .to_string()
            ],
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
}
