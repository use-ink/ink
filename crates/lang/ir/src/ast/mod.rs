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

//! Types and abstractions for ink! definitions that require custom syntax.
//!
//! # Note
//!
//! In general we try not to require any sort of custom non-standard Rust
//! syntax.
//!
//! At the time of this writing we currently only use this for the argument
//! parsing of ink! configuration header `#[ink(env = my::env::Types, etc...)]`
//! in order to be able to parse identifiers in `name = value` segments for
//! the `value` part.

mod attr_args;

pub use self::attr_args::{
    AttributeArgs,
    MetaNameValue,
    PathOrLit,
};
