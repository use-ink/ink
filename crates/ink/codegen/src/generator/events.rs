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

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
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
        // let event_base = self.generate_event_base();
        // let topic_guards = self.generate_topic_guards();
        // let topics_impls = self.generate_topics_impls();
        let event_structs = self.generate_event_structs();
        quote! {
            // #event_base
            // #( #topic_guards )*
            #( #event_structs )*
            // #( #topics_impls )*
        }
    }
}

impl<'a> Events<'a> {
    /// Generates all the user defined event struct definitions.
    fn generate_event_structs(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let attrs = event.attrs();
            // todo: should we just keep this attribute as part of attrs in the first
            // place?
            let anonymous_attr = event.anonymous.then(|| {
                quote_spanned!(span =>
                    #[ink(::anonymous)]
                )
            });
            quote_spanned!(span =>
                #( #attrs )*
                #[derive(::ink::Event, scale::Encode, scale::Decode)]
                #[cfg_attr(feature = "std", derive(ink::EventMetadata))]
                #anonymous_attr
                #event
            )
        })
    }
}
