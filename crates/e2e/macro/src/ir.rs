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
    config::E2EConfig,
    ir,
};
use proc_macro2::TokenStream as TokenStream2;

/// The End-to-End test with all required information.
pub struct InkE2ETest {
    /// The function which was annotated.
    pub item_fn: E2EFn,
    /// The specified configuration.
    pub config: E2EConfig,
}

/// The End-to-End test with all required information.
#[derive(derive_more::From)]
pub struct E2EFn {
    /// The function which was annotated.
    pub item_fn: syn::ItemFn,
}

impl InkE2ETest {
    /// Returns `Ok` if the test matches all requirements for an
    /// ink! E2E test definition.
    pub fn new(attrs: TokenStream2, input: TokenStream2) -> Result<Self, syn::Error> {
        let config = syn::parse2::<ink_ir::ast::AttributeArgs>(attrs)?;
        let e2e_config = ir::E2EConfig::try_from(config)?;
        let item_fn = syn::parse2::<syn::ItemFn>(input)?;
        let e2e_fn = E2EFn::from(item_fn);
        Ok(Self {
            item_fn: e2e_fn,
            config: e2e_config,
        })
    }
}
