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

//! The `ink_env` utilities used to interoperate with the contract executor.
//!
//! Mainly provides entities to work on a contract's storage
//! as well as high-level collections on top of those.
//! Also provides environmental utilities, such as storage allocators,
//! FFI to interface with FRAME contracts and a primitive blockchain
//! emulator for simple off-chain testing.

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    missing_docs,
    bad_style,
    bare_trait_objects,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates
)]

/// The capacity of the static buffer.
/// Usually set to 16 kB.
/// Can be modified by setting `INK_STATIC_BUFFER_SIZE` environmental variable.
#[const_env::from_env("INK_STATIC_BUFFER_SIZE")]
pub const BUFFER_SIZE: usize = 16384;

#[cfg(target_arch = "riscv64")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // In case the contract is build in debug-mode, we return the
    // panic message as a payload by triggering a contract revert.
    #[cfg(any(feature = "ink-debug", feature = "std"))]
    self::return_value(
        ReturnFlags::REVERT,
        &ink_prelude::format!("{}", info.message()).as_bytes(),
    );

    // If contract is compiled with `cargo contract --release`, it will
    // for efficiency reasons be build with `panic_immediate_abort`.
    // This panic handler will thus never be invoked.
    #[cfg(not(any(feature = "ink-debug", feature = "std")))]
    unreachable!(
        "contract in non-debug/non-std mode needs to be build with `panic_immediate_abort`"
    );
}

// This extern crate definition is required since otherwise rustc
// is not recognizing its allocator and panic handler definitions.
#[cfg(not(any(feature = "std", feature = "no-allocator")))]
extern crate ink_allocator;

mod api;
mod backend;
pub mod call;
mod engine;
mod error;
#[doc(hidden)]
pub mod event;
pub mod hash;

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
    error::{
        Error,
        Result,
    },
    event::{
        Event,
        TopicEncoder,
    },
    types::{
        AccountIdGuard,
        Balance,
        BlockNumber,
        CodecAsType,
        DefaultEnvironment,
        Environment,
        FromLittleEndian,
        Gas,
        Timestamp,
    },
};
pub use ink_primitives::{
    contract::{
        ContractEnv,
        ContractReference,
        ContractReverseReference,
    },
    reflect,
    reflect::{
        DecodeDispatch,
        DispatchError,
    },
    types,
};
#[doc(inline)]
pub use pallet_revive_uapi::{
    CallFlags,
    ReturnErrorCode,
    ReturnFlags,
};

/// A convenience type alias to the marker type representing the "default" ABI for calls.
///
/// # Note
///
/// The "default" ABI for calls is "ink", unless the ABI is set to "sol"
/// in the ink! project's manifest file (i.e. `Cargo.toml`).
#[cfg(not(ink_abi = "sol"))]
#[doc(hidden)]
pub type DefaultAbi = ink_primitives::abi::Ink;

/// A convenience type alias to the marker type representing the "default" ABI for calls.
///
/// # Note
///
/// The "default" ABI for calls is "ink", unless the ABI is set to "sol"
/// in the ink! project's manifest file (i.e. `Cargo.toml`).
#[cfg(ink_abi = "sol")]
#[doc(hidden)]
pub type DefaultAbi = ink_primitives::abi::Sol;
