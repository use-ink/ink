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
