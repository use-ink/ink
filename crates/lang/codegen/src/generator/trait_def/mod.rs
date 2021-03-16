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

mod concretizer;
mod definition;
mod short_call;
mod long_call;

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;

/// Generator to create the ink! storage struct and important trait impls.
#[derive(From)]
pub struct TraitDefinition<'a> {
    trait_def: &'a ir::InkTrait,
}

impl GenerateCode for TraitDefinition<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let trait_definition = self.generate_trait_definition();
        let trait_concretizer = self.generate_trait_concretizer();
        quote_spanned!(span =>
            #trait_definition
            const _: () = {
                #trait_concretizer
            };
        )
    }
}
