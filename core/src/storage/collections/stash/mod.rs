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

//! A stash collection.
//!
//! Provides O(1) random insertion, deletion and access of its elements.
//!
//! ## Guarantees and non-guarantees:
//!
//! 1. `Stash` is deterministic and keys do not depend on the inserted values.
//!    This means you can update two stashes in tandem and get the same keys
//!    back. This could be useful for, e.g., primary/secondary replication.
//! 2. Keys will always be less than the maximum number of items that have ever
//!    been present in the `Stash` at any single point in time. In other words,
//!    if you never store more than `n` items in a `Stash`, the stash will only
//!    assign keys less than `n`. You can take advantage of this guarantee to
//!    truncate the key from a `usize` to some smaller type.
//! 3. Except the guarantees noted above, you can assume nothing about key
//!    assignment or iteration order. They can change at any time.

#[cfg(test)]
mod tests;

mod impls;

pub use self::impls::{
    Iter,
    Stash,
    Values,
};
