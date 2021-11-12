// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

//! High-level collections used to manage storage entities in the persisted
//! contract storage.
//!
//! Users should generally use these collections in their contracts directly
//! or as building blocks for their collections and algorithms.

pub mod binary_heap;
pub mod bitstash;
pub mod bitvec;
pub mod hashmap;
pub mod smallvec;
pub mod stash;
pub mod vec;

#[doc(inline)]
pub use self::{
    binary_heap::BinaryHeap,
    bitstash::BitStash,
    bitvec::Bitvec,
    hashmap::HashMap,
    stash::Stash,
    vec::Vec,
};

#[doc(inline)]
pub use self::smallvec::SmallVec;

/// Extends the lifetime `'a` to the outliving lifetime `'b` for the given reference.
///
/// # Note
///
/// This interface is a bit more constraint than a simple
/// [transmute](`core::mem::transmute`) and therefore preferred
/// for extending lifetimes only.
///
/// # Safety
///
/// This function is `unsafe` because lifetimes can be extended beyond the
/// lifetimes of the objects they are referencing and thus potentially create
/// dangling references if not used carefully.
pub(crate) unsafe fn extend_lifetime<'a, 'b: 'a, T>(reference: &'a mut T) -> &'b mut T {
    core::mem::transmute::<&'a mut T, &'b mut T>(reference)
}
