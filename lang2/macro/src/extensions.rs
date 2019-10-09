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

//! Module to introduce extended functionality to types from other crates.

use proc_macro2::{
    Ident,
    Span,
};

/// Extensions for the `Ident` type.
pub trait IdentExt {
    /// Creates a new Ident from the given `str`.
    fn from_str<T: AsRef<str>>(s: T) -> Ident;
}

impl IdentExt for Ident {
    fn from_str<T: AsRef<str>>(s: T) -> Ident {
        Ident::new(s.as_ref(), Span::call_site())
    }
}

/// Extensions for the `FnArg` type.
pub trait FnArgExt {
    /// Returns the identifier of the function argument if it isn't a receiver
    /// (e.g. not `self`, `&self` or `&mut self`).
    fn ident(&self) -> Option<proc_macro2::Ident>;
    /// Returns the pattern and the type of the function argument if it isn't a receiver
    /// (e.g. not `self`, `&self` or `&mut self`).
    fn pat_type(&self) -> Option<&syn::PatType>;
}

impl FnArgExt for syn::FnArg {
    fn ident(&self) -> Option<proc_macro2::Ident> {
        match self {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(typed) => {
                match &*typed.pat {
                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                    _ => None,
                }
            }
        }
    }

    fn pat_type(&self) -> Option<&syn::PatType> {
        match self {
            syn::FnArg::Typed(typed) => Some(typed),
            _ => None,
        }
    }
}
