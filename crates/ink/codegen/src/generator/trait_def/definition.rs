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

//! Generates the ink! trait definition item.

use super::TraitDefinition;
use heck::ToLowerCamelCase as _;
use ir::Selector;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

impl<'a> TraitDefinition<'a> {
    fn generate_for_message(
        message: ir::InkTraitMessage<'a>,
        selector: Selector,
    ) -> TokenStream2 {
        let span = message.span();
        let attrs = message.attrs();
        let sig = message.sig();
        let ident = &sig.ident;
        let inputs = &sig.inputs;
        let cfg_attrs = message.get_cfg_attrs(span);
        let output = match &sig.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };
        let output_ident =
            format_ident!("{}Output", ident.to_string().to_lower_camel_case());

        let description_ident = format_ident!("__ink_{}_description", sig.ident);
        let is_mutable = message.receiver().is_ref_mut();
        let is_payable = message.ink_attrs().is_payable();
        let selector_bytes = selector.hex_lits();

        quote_spanned!(span =>
            /// Output type of the respective trait message.
            #(#cfg_attrs)*
            type #output_ident: ::ink::codegen::ImpliesReturn<#output>;

            #(#attrs)*
            fn #ident(#inputs) -> Self::#output_ident;

            #(#cfg_attrs)*
            #[inline(always)]
            fn #description_ident(&self) -> ::ink::reflect::MessageDescription {
                ::ink::reflect::MessageDescription::new(
                    #is_mutable,
                    #is_payable,
                    [ #( #selector_bytes ),* ]
                )
            }
        )
    }
}

impl TraitDefinition<'_> {
    pub(super) fn generate_trait_definition(&self) -> TokenStream2 {
        let item = self.trait_def.item();
        let span = item.span();
        let attrs = item.attrs();
        let ident = item.ident();
        let messages = item.iter_items().flat_map(|(item, selector)| {
            ir::InkTraitItem::filter_map_message(item)
                .map(|message| Self::generate_for_message(message, selector))
        });
        quote_spanned!(span =>
            #(#attrs)*
            pub trait #ident: ::ink::env::ContractEnv {
                /// Holds general and global information about the trait.
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __ink_TraitInfo: ::ink::codegen::TraitCallForwarder;

                #(#messages)*
            }
        )
    }
}
