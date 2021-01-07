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

//! The `ink_env` utilities used to interoperate with the contract executor.
//!
//! Mainly provides entities to work on a contract's storage
//! as well as high-level collections on top of those.
//! Also provides environmental utilities, such as storage allocators,
//! FFI to interface with SRML contracts and a primitive blockchain
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

#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // SAFETY: We only use this operation if we are guaranteed to be in Wasm32 compilation.
    //         This is used in order to make any panic a direct abort avoiding Rust's general
    //         panic infrastructure.
    unsafe {
        core::arch::wasm32::unreachable();
    }
}

// This extern crate definition is required since otherwise rustc
// is not recognizing its allocator and panic handler definitions.
#[cfg(not(feature = "std"))]
extern crate ink_allocator;

mod api;
mod arithmetic;
mod backend;
pub mod call;
pub mod chain_extension;
mod engine;
mod error;
pub mod hash;
#[doc(hidden)]
pub mod topics;
mod types;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "std", test, doc))]
#[doc(inline)]
pub use self::engine::off_chain::test_api as test;

use self::backend::{
    EnvBackend,
    TypedEnvBackend,
};
pub use self::{
    api::*,
    backend::ReturnFlags,
    error::{
        Error,
        Result,
    },
    topics::Topics,
    types::{
        AccountId,
        Clear,
        DefaultEnvironment,
        Environment,
        Hash,
        NoChainExtension,
    },
};
