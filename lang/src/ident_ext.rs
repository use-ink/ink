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

use proc_macro2::{
    Ident,
    Span,
};
use std::fmt::Display;

/// Utilities for operating on `Ident` instances.
pub trait IdentExt: Display {
    /// Creates a string out of the ident's name.
    fn to_owned_string(&self) -> String {
        format!("{}", self)
    }

    /// Creates a new Ident from the given `str`.
    fn from_str<T: AsRef<str>>(s: T) -> Ident {
        Ident::new(s.as_ref(), Span::call_site())
    }
}

impl IdentExt for Ident {}
