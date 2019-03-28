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

mod build;

use crate::hir;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
};

/// Generates code for the given contract.
///
/// # Note
///
/// This generates code for normal Wasm smart contract builds as
/// well as code for specialized `test` and `doc` targets.
pub fn generate_code(contract: &hir::Contract) -> TokenStream2 {
    let mut tokens = quote! {};
    build::generate_code(&mut tokens, contract);
    tokens
}
