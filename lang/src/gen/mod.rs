// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

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
