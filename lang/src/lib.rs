// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

#![recursion_limit = "128"]

extern crate proc_macro;

use quote::ToTokens;

#[proc_macro]
pub fn contract(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match contract_gen_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.into_token_stream().into(),
    }
}

#[macro_use]
mod errors;

mod api;
mod ast;
mod gen;
mod hir;
mod ident_ext;
mod parser;

#[cfg(test)]
mod tests;

use errors::Result;

/// Simple wrapper from `proc_macro` to `proc_macro2` and back again.
///
/// # Note
///
/// The actual `proc_macro` interface has to operate on `proc_macro::TokenStream`
/// but to keep this library testable we want to use only `proc_macro2::*` entities
/// internally.
fn contract_gen_impl(input: proc_macro::TokenStream) -> Result<proc_macro::TokenStream> {
    contract_gen_impl2(input.into()).map(Into::into)
}

/// Parses the given token stream as pDSL contract, performs some checks and returns
/// the corresponding contract as token stream.
pub(crate) fn contract_gen_impl2(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream> {
    let ast_contract = parser::parse_contract(input.clone())?;
    let hir_contract = hir::Contract::from_ast(&ast_contract)?;
    api::generate_api_description(&hir_contract);
    let tokens = gen::codegen(&hir_contract);
    Ok(tokens.into())
}
