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

//! Provide macros to simplify error reporting in procedural macros.

/// Returns a generated error result directly to the caller.
///
/// # Note
///
/// Takes some tokens that implement `ToTokens` trait in order to form a `Span`
/// and also takes a format string plus arbitrary many formatting parameters.
macro_rules! bail {
    ($($args:tt)*) => {
        return Err(format_err!($($args)*).into())
    }
}

/// Creates a macro error.
///
/// # Note
///
/// Takes some tokens that implement `ToTokens` trait in order to form a `Span`
/// and also takes a format string plus arbitrary many formatting parameters.
macro_rules! format_err {
    ($tokens:expr, $($msg:tt)*) => {
        syn::parse::Error::new_spanned(&$tokens, format_args!($($msg)*))
    }
}

/// Returns a generated error result directory to the caller.
///
/// # Note
///
/// Takes a concrete span as first argument followed by some format string plus
/// some additional format parameters.
macro_rules! bail_span {
    ($($args:tt)*) => {
        return Err(format_err_span!($($args)*).into())
    }
}

/// Creates a macro error.
///
/// # Note
///
/// Takes a concrete span as first argument followed by some format string plus
/// some additional format parameters.
macro_rules! format_err_span {
    ($span:expr, $($msg:tt)*) => {
        syn::parse::Error::new($span, format_args!($($msg)*))
    }
}
