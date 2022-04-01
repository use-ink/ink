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

use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

/// A checked ink! event definition.
#[derive(Debug, PartialEq, Eq)]
pub struct InkEventDefinition {
    // config: TraitDefinitionConfig,
    // item: InkItemTrait,
}

impl InkEventDefinition {
    /// Returns `Ok` if the input matches all requirements for an ink! trait definition.
    pub fn new(config: TokenStream2, input: TokenStream2) -> Result<Self> {
        todo!()
        // let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config)?;
        // let parsed_item = syn::parse2::<syn::ItemTrait>(input)?;
        // let config = TraitDefinitionConfig::try_from(parsed_config)?;
        // let item = InkItemTrait::new(&config, parsed_item)?;
        // Ok(Self { config, item })
    }
}