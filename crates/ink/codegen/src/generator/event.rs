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

use derive_more::From;
#[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
use ir::IsDocAttribute;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
#[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
use syn::Fields;

#[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
use crate::generator::sol;
use crate::GenerateCode;

/// Generates code for the event item.
#[derive(From, Copy, Clone)]
pub struct Event<'a> {
    /// The storage item to generate code for.
    item: &'a ir::Event,
}

impl GenerateCode for Event<'_> {
    /// Generates ink! event item code.
    fn generate_code(&self) -> TokenStream2 {
        let item = self.item.item();
        let anonymous = self
            .item
            .anonymous()
            .then(|| quote::quote! { #[ink(anonymous)] });
        let signature_topic = self
            .item
            .signature_topic_hex()
            .map(|hex_s| quote::quote! { #[ink(signature_topic = #hex_s)] });
        let cfg_attrs = self.item.get_cfg_attrs(item.span());

        #[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
        let sol_event_metadata = self.solidity_event_metadata();

        #[cfg(not(all(feature = "std", any(ink_abi = "sol", ink_abi = "all"))))]
        let sol_event_metadata = quote! {};

        quote::quote! (
            #( #cfg_attrs )*
            #[cfg_attr(feature = "std", derive(::ink::EventMetadata))]
            #[derive(::ink::Event)]
            #[::ink::scale_derive(Encode, Decode)]
            #anonymous
            #signature_topic
            #item

            #sol_event_metadata
        )
    }
}

impl Event<'_> {
    /// Generates Solidity ABI compatible metadata for ink! event.
    #[cfg(all(feature = "std", any(ink_abi = "sol", ink_abi = "all")))]
    fn solidity_event_metadata(&self) -> TokenStream2 {
        let item = self.item.item();
        let ident = &item.ident;
        let name = ident.to_string();
        let is_anonymous = self.item.anonymous();

        let fields = match &item.fields {
            Fields::Named(fields) => fields,
            Fields::Unnamed(_) | Fields::Unit => unreachable!("Expected named fields"),
        };
        let params = fields.named.iter().map(|field| {
            let ty = &field.ty;
            let sol_ty = sol::utils::sol_type(ty);
            let ident = field.ident.as_ref().expect("Expected a named field");
            let name = ident.to_string();
            let is_topic = field.attrs.iter().any(|attr| {
                let is_topic_arg = || {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("topic") {
                            Ok(())
                        } else {
                            Err(meta.error("Not a topic arg"))
                        }
                    })
                    .is_ok()
                };
                attr.path().is_ident("ink") && is_topic_arg()
            });
            let docs = field
                .attrs
                .iter()
                .filter_map(|attr| attr.extract_docs())
                .collect::<Vec<_>>()
                .join("\n");

            quote! {
                ::ink::metadata::sol::EventParamMetadata {
                    name: #name.into(),
                    ty: #sol_ty.into(),
                    is_topic: #is_topic,
                    docs: #docs.into(),
                }
            }
        });

        let docs = item
            .attrs
            .iter()
            .filter_map(|attr| attr.extract_docs())
            .collect::<Vec<_>>()
            .join("\n");

        quote! {
            const _: () = {
                // Register Solidity ABI compatible metadata function for event in distributed slice
                // for collecting all events referenced in the contract binary.
                #[::ink::linkme::distributed_slice(::ink::CONTRACT_EVENTS_SOL)]
                #[linkme(crate = ::ink::linkme)]
                static EVENT_METADATA_SOL: fn() -> ::ink::metadata::sol::EventMetadata = || {
                    ::ink::metadata::sol::EventMetadata {
                        name: #name.into(),
                        is_anonymous: #is_anonymous,
                        params: vec![ #( #params ),* ],
                        docs: #docs.into(),
                    }
                };
            };
        }
    }
}
