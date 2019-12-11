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

//! A BTreeMap collection.
//!
//! This implementation follows the algorithm used by the Rust
//! BTreeMap stdlib implementation. The Rust implementation is
//! in-memory, whereas this implementation uses the ink! storage
//! primitives (`SyncChunk`, etc.).

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod impls;
mod search;

pub use self::impls::{
    BTreeMap,
    Entry,
};
