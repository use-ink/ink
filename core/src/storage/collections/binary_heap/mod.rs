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

//! A binary heap collection.
//! The heap depends on `Ord` and is a max-heap by default. In order to
//! make it a min-heap implement the `Ord` trait explicitly on the type
//! which is stored in the heap.
//!
//! Provides `O(log(n))` push and pop operations.

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod duplex_sync_chunk;
mod impls;

pub use self::impls::{
    BinaryHeap,
    Iter,
    Values,
};
