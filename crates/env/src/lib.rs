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

/// The capacity of the static buffer.
/// Usually set to 16 kB.
/// Can be modified by setting `INK_STATIC_BUFFER_SIZE` environmental variable.
#[const_env::from_env("INK_STATIC_BUFFER_SIZE")]
pub const BUFFER_SIZE: usize = 16384;

#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[allow(unused_extern_crates)]
extern crate rlibc;

#[cfg(not(feature = "std"))]
#[allow(unused_variables)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // This code gets removed in release builds where the macro will expand into nothing.
    debug_print!("{}\n", info);

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            core::arch::wasm32::unreachable();
        } else if #[cfg(target_arch = "riscv32")] {
            // Safety: The unimp instruction is guaranteed to trap
            unsafe {
                core::arch::asm!("unimp");
                core::hint::unreachable_unchecked();
            }
        } else {
            core::compile_error!("ink! only supports wasm32 and riscv32");
        }
    }
}

// This extern crate definition is required since otherwise rustc
// is not recognizing its allocator and panic handler definitions.
#[cfg(not(any(feature = "std", feature = "no-allocator")))]
extern crate ink_allocator;

mod api;
mod backend;
pub mod call;
pub mod chain_extension;
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
    backend::{
        CallFlags,
        ReturnFlags,
    },
    error::{
        Error,
        Result,
    },
    event::Event,
    types::{
        AccountIdGuard,
        Balance,
        BlockNumber,
        CodecAsType,
        DefaultEnvironment,
        Environment,
        FromLittleEndian,
        Gas,
        NoChainExtension,
        Timestamp,
    },
};
use ink_primitives::Clear;
pub use ink_primitives::{
    contract::{
        ContractEnv,
        ContractReference,
        ContractReverseReference,
    },
    reflect,
    types,
};

cfg_if::cfg_if! {
    if #[cfg(any(feature = "ink-debug", feature = "std"))] {
        /// Required by the `debug_print*` macros below, because there is no guarantee that
        /// contracts will have a direct `ink_prelude` dependency. In the future we could introduce
        /// an "umbrella" crate containing all the `ink!` crates which could also host these macros.
        #[doc(hidden)]
        pub use ink_prelude::format;

        /// Appends a formatted string to the `debug_message` buffer if message recording is
        /// enabled in the contracts pallet and if the call is performed via RPC (**not** via an
        /// extrinsic). The `debug_message` buffer will be:
        ///  - Returned to the RPC caller.
        ///  - Logged as a `debug!` message on the Substrate node, which will be printed to the
        ///    node console's `stdout` when the log level is set to `-lruntime::contracts=debug`.
        ///
        /// # Note
        ///
        /// This depends on the `debug_message` interface which requires the
        /// `"pallet-contracts/unstable-interface"` feature to be enabled in the target runtime.
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => ($crate::debug_message(&$crate::format!($($arg)*)));
        }

        /// Appends a formatted string to the `debug_message` buffer, as per [`debug_print`] but
        /// with a newline appended.
        ///
        /// # Note
        ///
        /// This depends on the `debug_message` interface which requires the
        /// `"pallet-contracts/unstable-interface"` feature to be enabled in the target runtime.
        #[macro_export]
        macro_rules! debug_println {
            () => ($crate::debug_print!("\n"));
            ($($arg:tt)*) => (
                $crate::debug_print!("{}\n", $crate::format!($($arg)*));
            )
        }
    } else {
        #[macro_export]
        /// Debug messages disabled. Enable the `ink-debug` feature for contract debugging.
        macro_rules! debug_print {
            ($($arg:tt)*) => ();
        }

        #[macro_export]
        /// Debug messages disabled. Enable the `ink-debug` feature for contract debugging.
        macro_rules! debug_println {
            () => ();
            ($($arg:tt)*) => ();
        }
    }
}
