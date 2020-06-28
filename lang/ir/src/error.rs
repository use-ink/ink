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

pub trait ExtError {
    /// Returns `self` combined with the other error.
    fn into_combine(self, another: syn::Error) -> Self;
}

impl ExtError for syn::Error {
    fn into_combine(mut self, another: syn::Error) -> Self {
        self.combine(another);
        self
    }
}

/// Spawns a spanned [`syn::Error`] using the provided arguments.
///
/// # Parameters
///
/// - The first argument must implement [`quote::ToTokens`] in order to
///   infer a [`Span`](`proc_macro2::Span`).
/// - The second argument is a format string.
/// - The rest are format string arguments.
///
/// # Note
///
/// Takes some tokens that implement `ToTokens` trait in order to form a `Span`
/// and also takes a format string plus arbitrary many formatting parameters.
#[macro_export]
macro_rules! format_err {
    ($tokens:expr, $($msg:tt)*) => {
        ::syn::Error::new(
            <_ as syn::spanned::Spanned>::span(&$tokens),
            format_args!($($msg)*)
        )
    }
}

/// Spawns a [`syn::Error`] using a concrete span and the provided arguments.
///
/// # Parameters
///
/// - The first argument must be a concrete [`Span`](`proc_macro2::Span`) instance.
/// - The second argument is a format string.
/// - The rest are format string arguments.
///
/// # Note
///
/// Takes a concrete span as first argument followed by some format string plus
/// some additional format parameters.
#[macro_export]
macro_rules! format_err_span {
    ($span:expr, $($msg:tt)*) => {
        ::syn::Error::new($span, format_args!($($msg)*))
    }
}
