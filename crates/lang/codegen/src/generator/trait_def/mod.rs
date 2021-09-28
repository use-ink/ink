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

mod call_builder;
mod call_forwarder;
mod definition;
mod trait_registry;

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generator to create the ink! storage struct and important trait implementations.
#[derive(From, Copy, Clone)]
pub struct TraitDefinition<'a> {
    trait_def: &'a ir::InkTraitDefinition,
}

impl<'a> TraitDefinition<'a> {
    /// Appends the trait suffix to the string and forms an identifier.
    ///
    /// This appends the `_$NAME_$TRAIT_ID` string to the prefix string
    /// were `$NAME` is the non-unique name of the trait and `$TRAIT_ID`
    /// is the hex representation of the unique 4-byte trait identifier.
    fn append_trait_suffix(&self, prefix: &str) -> syn::Ident {
        let unique_id = self.trait_def.id().to_be_bytes();
        format_ident!(
            "__ink_{}_{}_0x{:X}{:X}{:X}{:X}",
            prefix,
            self.trait_def.item().ident(),
            unique_id[0],
            unique_id[1],
            unique_id[2],
            unique_id[3]
        )
    }

    /// Returns the span of the underlying ink! trait definition.
    fn span(&self) -> Span {
        self.trait_def.item().span()
    }
}

impl GenerateCode for TraitDefinition<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let span = self.trait_def.item().span();
        let trait_definition = self.generate_trait_definition();
        let trait_registry = self.generate_trait_registry_impl();
        let trait_call_builder = self.generate_call_builder();
        let trait_call_forwarder = self.generate_call_forwarder();
        let input_output_guards = self.generate_input_output_guards();
        quote_spanned!(span =>
            #trait_definition
            const _: () = {
                #trait_registry
                #trait_call_builder
                #trait_call_forwarder
                #input_output_guards
            };
        )
    }
}

impl TraitDefinition<'_> {
    /// Generates code to assert that ink! input and output types meet certain properties.
    fn generate_input_output_guards(&self) -> TokenStream2 {
        let storage_span = self.trait_def.item().span();
        let message_inout_guards = self
            .trait_def
            .item()
            .iter_items()
            .filter_map(|(impl_item, _)| impl_item.filter_map_message())
            .map(|message| {
                let message_span = message.span();
                let message_inputs = message.inputs().map(|input| {
                    let input_span = input.span();
                    let input_type = &*input.ty;
                    quote_spanned!(input_span=>
                        let _: () = ::ink_lang::type_check::identity_type::<
                            ::ink_lang::type_check::DispatchInput<#input_type>
                        >();
                    )
                });
                let message_output = message.output().map(|output_type| {
                    let output_span = output_type.span();
                    quote_spanned!(output_span=>
                        let _: () = ::ink_lang::type_check::identity_type::<
                            ::ink_lang::type_check::DispatchOutput<#output_type>
                        >();
                    )
                });
                quote_spanned!(message_span=>
                    #( #message_inputs )*
                    #message_output
                )
            });
        quote_spanned!(storage_span=>
            #( #message_inout_guards )*
        )
    }
}
