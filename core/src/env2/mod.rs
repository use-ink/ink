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

//! Environment definitions and access.
//!
//! Provides 5 different preferred ways to access the underlying environment.
//!
//! | Access Type | Optimized Access | Restricted Access | Checked Access | Dynamic Allocator |
//! |---|---|---|---|---|
//! | `SrmlEnv` or `TestEnv` | No | - | No | No |
//! | `EnvAccess` | Yes | `&self` | Yes (@rt) | No |
//! | `EnvAccessMut` | Yes | `&mut self` | Yes (@ct) | No |
//! | `DynEnv<EnvAccess>` | Yes | `&self` | Yes (@rt) | Yes |
//! | `DynEnv<EnvAccessMut>` | Yes | `&mut self` | Yes (@ct) | Yes |
//!
//! * - @rt: reads "at runtime"
//! * - @ct: reads "at compiletime"
//!
//! # Explanations
//!
//! - **Optimized Access:** Tries to reuse buffers and minimize allocations.
//! - **Restricted Access:** Restricts usage for certain message types, e.g. only for `&mut self` messages.
//! - **Checked Access:** Checks certain accesses to the environment for obvious failures.
//! - **Dynamic Allocator:** Additionally provides an out-of-box dynamic allocator and an interface to
//!                          allocate and instantiate dynamic storage objects.
//!
//! # Note
//!
//! - If your contract uses dynamic allocations prefer using `DynEnvAccess` or `DynEnvAccessMut`.
//! - For `&self` messages prefer using `EnvAccess` or `DynEnvAccess`.
//! - For `&mut self` messages prefer using `EnvAccessMut` or `DynEnvAccessMut`.
//! - Direct access to `SrmlEnv` or `TestEnv` is always the least optimal solution and generally not preferred.

mod buffer_arena;
pub mod call;
mod dyn_env;
mod env_access;
mod error;
pub mod property;
mod traits;
pub mod types;
pub mod utils;

use cfg_if::cfg_if;

/// Error definitions specific to environment accesses.
pub mod errors {
    pub use super::error::{
        CallError,
        CreateError,
    };
}

cfg_if! {
    if #[cfg(feature = "test-env")] {
        pub mod test;
        /// The currently chosen environmental implementation.
        ///
        /// When compiling for Wasm and Substrate this refers to `SrmlEnv` and
        /// when compiling for off-chain testing this refer to `TestEnv`.
        ///
        /// This configuration compiled for off-chain testing.
        pub type EnvImpl<T> = self::test::TestEnv<T>;
    } else {
        mod srml;
        pub use self::srml::{
            RetCode,
        };
        /// The currently chosen environmental implementation.
        ///
        /// When compiling for Wasm and Substrate this refers to `SrmlEnv` and
        /// when compiling for off-chain testing this refer to `TestEnv`.
        ///
        /// This configuration compiled as Wasm for Substrate.
        pub type EnvImpl<T> = self::srml::SrmlEnv<T>;
    }
}

#[cfg(not(feature = "std"))]
pub use self::buffer_arena::GlobalBufferArena;

pub use self::{
    buffer_arena::{
        BufferArena,
        BufferRef,
        BUFFER_ARENA,
    },
    dyn_env::DynEnv,
    env_access::{
        AccessEnv,
        EmitEvent,
        EnvAccess,
        EnvAccessMut,
    },
    error::{
        Error,
        Result,
    },
    traits::{
        Env,
        EnvTypes,
        GetProperty,
        SetProperty,
        Topics,
    },
    types::DefaultSrmlTypes,
};
