// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

/// Creates a [`syn::Error`] with the format message and infers the
/// [`Span`](`proc_macro2::Span`) using [`ToTokens`](`quote::ToTokens`).
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
/// On stable Rust this might yield higher quality error span information to the user
/// than [`format_err`].
/// - Source:
/// [`syn::Error::new_spanned`](https://docs.rs/syn/1.0.33/syn/struct.Error.html#method.new_spanned)
/// - Tracking issue: [`#54725`](https://github.com/rust-lang/rust/issues/54725)
#[macro_export]
macro_rules! format_err_spanned {
    ($tokens:expr, $($msg:tt)*) => {
        ::syn::Error::new_spanned(
            &$tokens,
            format_args!($($msg)*)
        )
    }
}

/// Creates a [`syn::Error`] with the format message and infers the
/// [`Span`](`proc_macro2::Span`) using [`Spanned`](`syn::spanned::Spanned`).
///
/// # Parameters
///
/// - The first argument must be a type that implements [`syn::spanned::Spanned`].
/// - The second argument is a format string.
/// - The rest are format string arguments.
///
/// # Note
///
/// On stable Rust this might yield worse error span information to the user
/// than [`format_err_spanned`].
/// - Source:
/// [`syn::Error::new_spanned`](https://docs.rs/syn/1.0.33/syn/struct.Error.html#method.new_spanned)
/// - Tracking issue: [`#54725`](https://github.com/rust-lang/rust/issues/54725)
#[macro_export]
macro_rules! format_err {
    ($spanned:expr, $($msg:tt)*) => {
        ::syn::Error::new(
            <_ as ::syn::spanned::Spanned>::span(&$spanned),
            format_args!($($msg)*)
        )
    }
}
