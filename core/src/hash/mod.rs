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

//! High-level, built-in and efficient cryptographic hashing routines.

mod accumulator;
mod builder;
pub mod hasher;

pub use self::{
    accumulator::{
        Accumulator,
        Wrap,
    },
    builder::{
        HashBuilder,
        NoAccumulator,
    },
};

/// SHA2 256-bit hash builder.
pub type Sha2x256<S = NoAccumulator> = HashBuilder<hasher::Sha2x256Hasher, S>;
/// KECCAK 256-bit hash builder.
pub type Keccak256<S = NoAccumulator> = HashBuilder<hasher::Keccak256Hasher, S>;
/// BLAKE2 256-bit hash builder.
pub type Blake2x256<S = NoAccumulator> = HashBuilder<hasher::Blake2x256Hasher, S>;
/// BLAKE2 128-bit hash builder.
pub type Blake2x128<S = NoAccumulator> = HashBuilder<hasher::Blake2x128Hasher, S>;
