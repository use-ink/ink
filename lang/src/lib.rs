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

#![recursion_limit = "256"]

extern crate proc_macro;

#[macro_use]
mod error;

mod ast;
mod gen;
mod hir;
mod ident_ext;
mod parser;

#[cfg(test)]
mod tests;

mod contract;

use proc_macro::TokenStream;

#[proc_macro]
pub fn contract(input: TokenStream) -> TokenStream {
    contract::generate(input.into()).into()
}

#[cfg(test)]
pub use contract::generate_or_err;
