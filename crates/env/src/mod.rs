// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

//! Environmental interface. (version 3)
//!
//! This is the interface with which a smart contract is able to communicate
//! with the outside world through its sandbox boundaries.

mod api;
mod arithmetic;
mod backend;
pub mod call;
mod engine;
mod error;
pub mod hash;
mod types;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "std", test, doc))]
#[doc(inline)]
pub use self::engine::off_chain::test_api as test;

use self::backend::{
    Env,
    TypedEnv,
};
pub use self::{
    api::*,
    backend::ReturnFlags,
    error::{
        EnvError,
        Result,
    },
    hash::{
        Blake2x128,
        Blake2x256,
        CryptoHash,
        HashOutput,
        Keccak256,
        Sha2x256,
    },
    types::{
        AccountId,
        Clear,
        DefaultEnvTypes,
        EnvTypes,
        Hash,
        Topics,
    },
};
