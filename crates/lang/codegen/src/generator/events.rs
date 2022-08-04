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
use ir::Event;
use proc_macro2::{
    Span,
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
        let event_base = self.generate_event_base();
        let topic_guards = self.generate_topic_guards();
        let event_defs = self.generate_event_definitions(); // todo: call into shared event_def code for inline events
        quote! {
            #event_base
            #( #topic_guards )*
            #( #event_defs )*
        }
    }
}

impl<'a> Events<'a> {
    /// Generates the base event enum that comprises all user defined events.
    /// All emitted events are converted into a variant of this enum before being
    /// serialized and emitted to apply their unique event discriminant (ID).
    ///
    /// # Developer Note
    ///
    /// The `__ink_dylint_EventBase` config attribute is used here to convey the
    /// information that the generated enum is an ink! event to `dylint`.
    fn generate_event_base(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        let event_idents = self
            .contract
            .module()
            .events()
            .map(|event| event.ident())
            .collect::<Vec<_>>();
        let base_event_ident =
            proc_macro2::Ident::new("__ink_EventBase", Span::call_site());
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(::scale::Encode, ::scale::Decode)]
            #[cfg(not(feature = "__ink_dylint_EventBase"))]
            pub enum #base_event_ident {
                #( #event_idents(#event_idents), )*
            }

            const _: () = {
                impl ::ink_lang::reflect::ContractEventBase for #storage_ident {
                    type Type = #base_event_ident;
                }
            };

            #(
                const _: () = {
                    impl From<#event_idents> for #base_event_ident {
                        fn from(event: #event_idents) -> Self {
                            Self::#event_idents(event)
                        }
                    }
                };
            )*

            const _: () = {
                pub enum __ink_UndefinedAmountOfTopics {}
                impl ::ink_env::topics::EventTopicsAmount for __ink_UndefinedAmountOfTopics {
                    const AMOUNT: usize = 0;
                }

                impl ::ink_env::Topics for #base_event_ident {
                    type RemainingTopics = __ink_UndefinedAmountOfTopics;

                    fn topics<E, B>(
                        &self,
                        builder: ::ink_env::topics::TopicsBuilder<::ink_env::topics::state::Uninit, E, B>,
                    ) -> <B as ::ink_env::topics::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink_env::Environment,
                        B: ::ink_env::topics::TopicsBuilderBackend<E>,
                    {
                        match self {
                            #(
                                Self::#event_idents(event) => {
                                    <#event_idents as ::ink_env::Topics>::topics::<E, B>(event, builder)
                                }
                            )*
                        }
                    }
                }
            };
        }
    }

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
            match event {
                Event::Inline(event_def) => {
                    let event_def_gen = EventDefinition::from(event_def);
                    event_def_gen.generate_code()
                }
                Event::Imported(imported_event) => {
                    // todo: hook up to base event and figure out metadata
                    let item_type = &imported_event.item;
                    quote! { #item_type }
                }
            }
        })
    }
}
