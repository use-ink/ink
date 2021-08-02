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

//! The `ink_storage` utilities used to manipulate and organize contract storage.
//!
//! Mainly provides entities to work on a contract's storage
//! as well as high-level collections on top of those.
//! Also provides environmental utilities, such as storage allocators,
//! FFI to interface with FRAME contracts and a primitive blockchain
//! emulator for simple off-chain testing.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    missing_docs,
    bad_style,
    bare_trait_objects,
    const_err,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates
)]

#[cfg(all(test, feature = "std", feature = "ink-fuzz-tests"))]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod alloc;
pub mod collections;
pub mod lazy;
mod memory;
mod pack;
pub mod traits;

#[cfg(test)]
mod hashmap_entry_api_tests;

#[cfg(test)]
mod test_utils;

#[doc(inline)]
pub use self::{
    alloc::Box,
    collections::Vec,
    lazy::Lazy,
    memory::Memory,
    pack::Pack,
};
