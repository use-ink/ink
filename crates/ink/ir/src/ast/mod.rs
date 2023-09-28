// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Types and abstractions for ink! definitions that require custom
//! [`syn::parse::Parse`] implementations.
//!
//! # Note
//!
//! In general we do not require any sort of custom non-standard Rust syntax.
//!
//! However, because the Rust attribute grammar is very flexible,
//! custom [`syn::parse::Parse`] implementations are typically required
//! for parsing structured arguments from attribute syntax that doesn't
//! exactly match the
//! ["meta item" attribute syntax](https://doc.rust-lang.org/reference/attributes.html#meta-item-attribute-syntax)
//!  used by most
//! ["built-in" Rust attributes](https://doc.rust-lang.org/reference/attributes.html#built-in-attributes-index)
//! for which [`syn::Meta`] (and its related variant types) can be used directly.
//!
//! At the time of this writing, ink! attribute argument syntax deviates from
//! ["meta item" attribute syntax](https://doc.rust-lang.org/reference/attributes.html#meta-item-attribute-syntax)
//! by:
//! - allowing the `impl` keyword in "meta item" paths
//! (i.e. `#[ink(impl)]` which is a deviation from the
//! [simple path](https://doc.rust-lang.org/reference/paths.html#simple-paths)
//! grammar).
//! - allowing the `@` symbol as a `value` in `name-value` pairs
//! (i.e. `#[ink(selector = @)]` which is a deviation from the
//! [expression](https://doc.rust-lang.org/reference/expressions.html)
//! grammar followed by the `value` part).
//!
//! NOTE: Underscore (`_`) values in `name-value` pairs
//! (e.g. `#[ink(selector = _)]`) are technically allowed by
//! the "meta item" attribute syntax as they can be interpreted as
//! [underscore expressions](https://doc.rust-lang.org/reference/expressions/underscore-expr.html)
//! (same for path values - e.g `#[ink(env = my::env::Types)]` -
//! which are valid
//! [path expressions](https://doc.rust-lang.org/reference/expressions/path-expr.html)
//! in "meta item" attribute syntax).

mod attr_args;
mod meta;

pub use self::{
    attr_args::AttributeArgs,
    meta::{
        Meta,
        MetaNameValue,
        MetaValue,
        Symbol,
    },
};
