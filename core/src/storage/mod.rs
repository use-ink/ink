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

//! Provides low-level primitives to operate on contract storage.
//!
//! The following table lists all kinds of guarantees and what they provide for their users.
//!
//! ## Guarantees
//!
//! | Guarantee    | Description |
//! |:-------------|:------------|
//! | `Owned`      | Disallows aliasing between different kinds of these primitives. |
//! | `Typed`      | Automatically encodes and decodes the stored entity. |
//! | `Opt. Reads` | Tries to avoid unnecessary reads to the storage. |
//! | `Mutable`    | Allows inplace mutation of the stored entity. |
//! | `Safe Load`  | Guarantees to always have a valid element stored in the associated contract storage slot. |
//!
//! ## Structure
//!
//! ### Key
//!
//! The bare metal abstraction.
//!
//! It can be compared to a C-style raw void pointer that points to arbitrary memory.
//! `Key` allows arbitrary pointer-arithmetic and provides absolutely no guarantees to its users.
//!
//! ### Cells
//!
//! There are many different cell types.
//!
//! In essence all `Cell` types guarantee anti-aliased memory access.
//!
//! ### Entities
//!
//! The highest-level abstraction concerning contract storage primitive.
//!
//! They provide the most guarantees and should be preferred over the other
//! primitive types if possible.
//!
//! ## Primitives
//!
//! These are the new primitives for contract storage access and their provided guarantees.
//!
//! | Primitive   | Owned | Typed | Opt. Reads | Mutable | Safe Load |
//! |:-----------:|:-----:|:-----:|:----------:|:-------:|:---------:|
//! | `Key`       | No    | No    | No         | No      | No        |
//! | `RawCell`   | Yes   | No    | No         | No      | No        |
//! | `TypedCell` | Yes   | Yes   | No         | No      | No        |
//! | `SyncCell`  | Yes   | Yes   | Yes        | Yes     | No        |
//!
//! ## Chunks
//!
//! Chunks allow to operate on a collection of cells.
//! They can be compared to an array or a vector of cells.
//! There is one chunked version of every cell type and it provides the same guarantees.
//!
//! ### Kinds
//!
//! - `RawChunk`
//! - `TypedChunk`
//! - `SyncChunk`

pub mod alloc;
mod key;
mod non_clone;

pub mod cell;
pub mod chunk;
mod collections;
mod flush;
mod value;

use self::non_clone::NonCloneMarker;

pub use self::{
    collections::{
        binary_heap::{
            self,
            BinaryHeap,
        },
        bitvec::{
            self,
            BitVec,
        },
        hash_map::{
            self,
            HashMap,
        },
        stash::{
            self,
            Stash,
        },
        vec::{
            self,
            Vec,
        },
    },
    flush::Flush,
    key::{
        Key,
        KeyDiff,
    },
};

#[doc(inline)]
pub use self::alloc::Allocator;

#[doc(inline)]
pub use self::value::Value;
