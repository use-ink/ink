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
    match contract_gen_inner(input) {
        Ok(tokens) => tokens,
        Err(err) => err.into_token_stream().into(),
    }
}

#[macro_use]
mod errors;

mod ast;
mod gen;
mod hir;
mod ident_ext;
mod parser;

use errors::Result;

fn contract_gen_inner(input: proc_macro::TokenStream) -> Result<proc_macro::TokenStream> {
    let ast_contract = parser::parse_contract(input.clone())?;
    let hir_contract = hir::Contract::from_ast(&ast_contract)?;
    // gen::gir::generate(&hir_program)?;
    let tokens = gen::codegen(&hir_contract);
    Ok(tokens.into())
    // Ok(proc_macro::TokenStream::new())
}
