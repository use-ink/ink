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

mod call_builder;
mod call_forwarder;
mod definition;
mod message_builder;
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

/// Generator to create the ink! storage struct and important trait implementations.
#[derive(From, Copy, Clone)]
pub struct TraitDefinition<'a> {
    trait_def: &'a ir::InkTraitDefinition,
}

impl TraitDefinition<'_> {
    /// Appends the trait suffix to the string and forms an identifier.
    ///
    /// This appends the `_$NAME_$TRAIT_ID` string to the prefix string
    /// were `$NAME` is the non-unique name of the trait and `$TRAIT_ID`
    /// is the hex representation of the unique 4-byte trait identifier.
    fn append_trait_suffix(&self, prefix: &str) -> syn::Ident {
        format_ident!("__ink_{}{}", prefix, self.trait_def.item().ident(),)
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
        let trait_message_builder = self.generate_message_builder();
        let trait_call_builder = self.generate_call_builder();
        let trait_call_forwarder = self.generate_call_forwarder();
        quote_spanned!(span =>
            #trait_definition
            const _: () = {
                #trait_registry
                #trait_message_builder
                #trait_call_builder
                #trait_call_forwarder
            };
        )
    }
}
