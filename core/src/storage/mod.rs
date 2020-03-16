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
//! - `TypedChunk`
//! - `SyncChunk`

pub mod alloc;
pub mod cell;
pub mod chunk;
mod collections;
mod flush;
mod value;

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
        btree_map::{
            self,
            BTreeMap,
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
};

#[doc(inline)]
pub use self::alloc::Allocator;

#[doc(inline)]
pub use self::value::Value;
