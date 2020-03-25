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

//! The `ink_core` utilities used by all ink! smart contracts.
//!
//! Mainly provides entities to work on a contract's storage
//! as well as high-level collections on top of those.
//! Also provides environmental utilities, such as storage allocators,
//! FFI to interface with SRML contracts and a primitive blockchain
//! emulator for simple off-chain testing.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    bad_style,
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

// This extern crate definition is required since otherwise rustc
// is not recognizing its allocator and panic handler definitions.
#[cfg(not(feature = "std"))]
extern crate ink_alloc;

pub mod env;
pub mod hash;
pub mod storage;

// Needed for derive macros of `core/derive` sub crate.
pub(crate) use crate as ink_core;
