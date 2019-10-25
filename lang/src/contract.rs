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

use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

#[cfg(feature = "ink-generate-abi")]
use crate::old_abi;
use crate::{
    gen,
    hir,
    parser,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

pub fn generate_or_err(input: TokenStream2) -> Result<TokenStream2> {
    let ast_contract = parser::parse_contract(input)?;
    let hir_contract = hir::Contract::from_ast(&ast_contract)?;
    #[cfg(feature = "ink-generate-abi")]
    old_abi::generate_old_abi(&hir_contract)?;
    let tokens = gen::generate_code(&hir_contract);
    Ok(tokens)
}
