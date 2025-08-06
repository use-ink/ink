// Copyright (C) Use Ink (UK) Ltd.
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

//! Utility types and definitions used by the ink! codegen.

mod identity_type;
mod same_type;

pub use self::{
    identity_type::consume_type,
    same_type::IsSameType,
};

/// Compile-time `format!`-like macro that returns `&'static str`.
///
/// The first argument is a format string which can be a `const` expression
/// (i.e. doesn't have to be a literal). Like `format!` the format string
/// can contain `{}`s which are replaced by the additional parameters
/// which must all be `const` expressions.
///
/// See [`const_format::formatc`][formatc] for details.
///
/// [formatc]: https://docs.rs/const_format/latest/const_format/macro.formatc.html
pub use const_format::formatc as const_format;

/// Compile-time `concat!`-like macro that accepts `const` expressions as arguments
/// (i.e. not just literals) and returns `&'static str`.
///
/// See [`const_format::concatc`][concatc] for details.
///
/// [concatc]: https://docs.rs/const_format/latest/const_format/macro.concatc.html
pub use const_format::concatc as const_concat;
