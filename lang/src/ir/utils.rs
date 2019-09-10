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

use proc_macro2::Span;
use syn::{
    Result,
    parse::{
        Parse,
        ParseStream,
    },
};

/// An unsuffixed integer literal: `0` or `42` or `1337`
#[derive(Debug)]
pub struct UnsuffixedLitInt {
    pub(crate) lit_int: syn::LitInt,
}

impl UnsuffixedLitInt {
    /// Returns the unsuffixed literal integer.
    pub fn lit_int(&self) -> &syn::LitInt {
        &self.lit_int
    }

    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.lit_int.span()
    }
}

impl Parse for UnsuffixedLitInt {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit_int: syn::LitInt = input.parse()?;
        if lit_int.suffix() != "" {
            bail!(lit_int, "integer suffixes are not allowed here",)
        }
        Ok(Self { lit_int })
    }
}
