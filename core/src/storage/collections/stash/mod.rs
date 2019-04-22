// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

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

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod impls;

pub use self::impls::{
    Iter,
    Stash,
    Values,
};
