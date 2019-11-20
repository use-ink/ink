// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

mod abi;
mod as_dependency;
mod build;
mod doc;
mod test;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_str,
    Expr,
};

use crate::hir;

/// Generates code for the given contract.
///
/// # Note
///
/// This generates code for normal Wasm smart contract builds as
/// well as code for specialized `test` and `doc` targets.
pub fn generate_code(contract: &hir::Contract) -> TokenStream2 {
    let mut tokens = quote! {};
    build::generate_code(&mut tokens, contract);
    doc::generate_code(&mut tokens, contract);
    test::generate_code(&mut tokens, contract);
    abi::generate_code(&mut tokens, contract);
    as_dependency::generate_code(&mut tokens, contract);
    tokens
}

/// The function is required because syn doesn't provide a
/// `ToTokens` implementation for `[u8; 4]`.
fn selector_to_expr(selector: [u8; 4]) -> Expr {
    let selector = format!(
        "[{}, {}, {}, {}]",
        selector[0], selector[1], selector[2], selector[3]
    );
    parse_str::<syn::Expr>(&selector).expect("failed to parse selector")
}
