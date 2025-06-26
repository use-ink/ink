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
/// The first argument is a format string which can be a `String` (i.e. doesn't have
/// to be a literal). Like `format!` the format string can contain `{}`s which are
/// replace by the additional parameters which must all be constant expressions.
///
/// See [`const_format::formatc`] for details.
pub use const_format::formatc as const_format;
