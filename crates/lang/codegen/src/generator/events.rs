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

use crate::{
    generator::EventDefinition,
    GenerateCode,
};
use derive_more::From;
use proc_macro2::{
    TokenStream as TokenStream2,
};
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generates code for the ink! event structs of the contract.
#[derive(From)]
pub struct Events<'a> {
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(Events);

impl GenerateCode for Events<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.module().events().next().is_none() {
            // Generate no code in case there are no event definitions.
            return TokenStream2::new()
        }
        let topic_guards = self.generate_topic_guards();
        let event_defs = self.generate_event_definitions(); // todo: call into shared event_def code for inline events
        quote! {
            #( #topic_guards )*
            #( #event_defs )*
        }
    }
}

impl<'a> Events<'a> {
    /// Generate checks to guard against too many topics in event definitions.
    fn generate_topics_guard(&self, event: &ir::Event) -> TokenStream2 {
        let span = event.span();
        let storage_ident = self.contract.module().storage().ident();
        let event_ident = event.ident();
        let max_len_topics = quote_spanned!(span=>
            <<#storage_ident as ::ink_lang::reflect::ContractEnv>::Env
                as ::ink_env::Environment>::MAX_EVENT_TOPICS
        );
        quote_spanned!(span=>
            const _: () = ::ink_lang::codegen::utils::consume_type::<
                ::ink_lang::codegen::EventRespectsTopicLimit<
                    #event_ident,
                    { #max_len_topics },
                >
            >();
        )
    }

    /// Generates the guard code that protects against having too many topics defined on an ink! event.
    fn generate_topic_guards(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let topics_guard = self.generate_topics_guard(event);
            quote_spanned!(span =>
                #topics_guard
            )
        })
    }

    /// Generates the `Topics` trait implementations for the user defined events.
    fn generate_event_definitions(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.module().events().map(move |event| {
            let event_def_gen = EventDefinition::from(&event.event_def);
            event_def_gen.generate_code()
        })
    }
}
