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
        let event_items = self.generate_event_items();
        quote! {
            #( #event_items )*
        }
    }
}

impl<'a> Events<'a> {
    /// Generates all the user defined event struct definitions.
    fn generate_event_items(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.module().events().map(move |event| {
            let span = event.span();
            let attrs = event.attrs();
            // add back the `#[ink(anonymous)]` attribute if it was present, for parsing
            // by the derive macros.
            let anonymous_attr = event.anonymous.then(|| {
                quote_spanned!(span =>
                    #[ink(anonymous)]
                )
            });
            quote_spanned!(span =>
                #( #attrs )*
                #[derive(::ink::Event, ::scale::Encode, ::scale::Decode)]
                #[cfg_attr(feature = "std", derive(::ink::EventMetadata))]
                #anonymous_attr
                #event
            )
        })
    }
}
