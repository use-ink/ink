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

use crate::ir::idents_lint;
use core::convert::TryFrom;
use proc_macro2::TokenStream as TokenStream2;

/// The ink! test with all required information.
pub struct InkTest {
    /// The function which was annotated.
    pub item_fn: syn::ItemFn,
}

impl TryFrom<syn::ItemFn> for InkTest {
    type Error = syn::Error;

    fn try_from(item_fn: syn::ItemFn) -> Result<Self, Self::Error> {
        idents_lint::ensure_no_ink_identifiers(&item_fn)?;
        Ok(Self { item_fn })
    }
}

impl InkTest {
    /// Returns `Ok` if the trait matches all requirements for an ink! trait definition.
    pub fn new(attr: TokenStream2, input: TokenStream2) -> Result<Self, syn::Error> {
        if !attr.is_empty() {
            return Err(format_err_spanned!(
                attr,
                "unexpected attribute input for ink! trait definition"
            ))
        }
        let item_fn = syn::parse2::<syn::ItemFn>(input)?;
        InkTest::try_from(item_fn)
    }
}
