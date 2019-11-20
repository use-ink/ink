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
