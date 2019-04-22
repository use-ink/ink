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
use quote::ToTokens;
use std::result::Result as StdResult;
pub use syn::parse::Error as SynError;

macro_rules! bail {
    ($($args:tt)*) => {
        return Err(format_err!($($args)*).into())
    }
}

macro_rules! format_err {
    ($tokens:expr, $($msg:tt)*) => {
        match &$tokens {
            t => {
                syn::parse::Error::new_spanned(t, format_args!($($msg)*))
            }
        }
    }
}

/// A collection of errors.
///
/// # Note
///
/// This is used to allow for reporting multiple errors at the same time.
#[derive(Debug)]
pub struct Errors {
    errors: Vec<SynError>,
}

impl From<SynError> for Errors {
    fn from(err: SynError) -> Errors {
        Errors { errors: vec![err] }
    }
}

impl From<Vec<Errors>> for Errors {
    fn from(err: Vec<Errors>) -> Errors {
        let result = err.into_iter().flat_map(|v| v.errors).collect::<Vec<_>>();
        assert!(!result.is_empty());
        Errors { errors: result }
    }
}

/// Used to create a TokenStream from a list of errors
impl ToTokens for Errors {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for item in self.errors.iter() {
            item.to_compile_error().to_tokens(tokens);
        }
    }
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for err in &self.errors {
            err.fmt(f)?;
        }
        Ok(())
    }
}

/// Result type alias for an error type which allows for accumulating errors.
pub type Result<T> = StdResult<T, Errors>;
