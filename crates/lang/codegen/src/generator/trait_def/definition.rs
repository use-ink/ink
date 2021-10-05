// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
use heck::CamelCase as _;
use ir::HexLiteral;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

impl<'a> TraitDefinition<'a> {
    fn generate_for_message(message: ir::InkTraitMessage<'a>) -> TokenStream2 {
        let span = message.span();
        let attrs = message.attrs();
        let sig = message.sig();
        let ident = &sig.ident;
        let inputs = &sig.inputs;
        let output = match &sig.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };
        let output_ident = format_ident!("{}Output", ident.to_string().to_camel_case());
        quote_spanned!(span =>
            /// Output type of the respective trait message.
            type #output_ident: ::ink_lang::codegen::ImpliesReturn<#output>;

            #(#attrs)*
            fn #ident(#inputs) -> Self::#output_ident;
        )
    }
}

impl TraitDefinition<'_> {
    pub(super) fn generate_trait_definition(&self) -> TokenStream2 {
        let item = self.trait_def.item();
        let span = item.span();
        let attrs = item.attrs();
        let ident = item.ident();
        let unique_trait_id = self.trait_def.id().hex_padded_suffixed();
        let messages = item
            .iter_items()
            .map(|(item, _)| item)
            .flat_map(ir::InkTraitItem::filter_map_message)
            .map(Self::generate_for_message);
        quote_spanned!(span =>
            #(#attrs)*
            pub trait #ident: ::ink_lang::reflect::ContractEnv {
                /// Holds general and global information about the trait.
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __ink_TraitInfo: ::ink_lang::TraitUniqueId
                    + ::ink_lang::TraitCallForwarder
                    + ::ink_lang::codegen::TraitImplementedById<#unique_trait_id>;

                #(#messages)*
            }
        )
    }
}
