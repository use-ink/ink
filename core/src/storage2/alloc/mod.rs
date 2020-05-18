// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! The default dynamic storage allocator.
//!
//! Allows to allocate storage cells in a dynamic fashion.
//! This is important if users want to combine types of varying storage
//! footprints. For example, dynamic allocations are required whenever
//! a user wants to use a storage collection (e.g. `storage::Vec`) in
//! another storage collection: `storage::Vec<storage::Vec<T>>`
//!
//! # Simplification
//!
//! The contracts pallet is using 256 bit keys for identifying storage cells.
//! This implies a storage space of 2^256 cells which is big enough to say that
//! there are probably never going to happen collisions anywhere at any time
//! if keys are chosen randomly. Using the built-in crypto hashers on unique
//! input we can be sure that there are never going to be collisions in this
//! space of 2^256 cells.
//!
//! This way we can reduce the problem of finding another region in our storage
//! that fits certain requirements (e.g. a minimum size) to the problem of
//! finding another uniform slot. Since we are on 32-bit WebAssembly we have
//! memory limitations that makes it impractical to have more than 2^32 dynamic
//! allocated entities and so we can create another limitation for having a
//! total of 2^32 dynamic allocations at any point in time.
//! This enables us to have 32-bit keys instead of 256-bit keys.
//!
//! We can convert such 32-bit keys (represented by e.g. a `u32`) into 256-bit
//! keys by using one of the built-in crypto hashes that has a 256-bit output,
//! e.g. KECCAK, SHA2 or BLAKE2. For technical reasons we should prepend the
//! bytes of the 32-bit key by some unique byte sequence, e.g.:
//! ```no_compile
//! let key256 = blake2x256(b"DYNAMICALLY ALLOCATED", bytes(key32));
//! ```
//!
//! # Internals
//!
//! As described in [# Simplification] there are 2^32 possible uniform dynamic
//! allocations available. For each such slot the dynamic allocator stores via
//! a single bit in a bitvector if that slot is free or occupied.
//! This bitvector is called the `free` list.
//! However, searching in this `free` list for a 0 bit and thus a free slot
//! for a dynamic allocation would mean that for every 256 consecutively
//! occupied dynamic allocations there was a contract storage lookup required.
//! This might seem a lot but given that there could be thousands or
//! tens of thousands of dynamic allocations at any given time this might not scale
//! well.
//! For the reason of improving scalability we added another vector: the
//! so-called `set_bits` vector.
//! In this vector every `u8` element densely stores the number of set bits
//! (bits that are `1` or `true`) for each 256-bit package in the `free` list.
//! (Note that the `free` list is organized in 256-bit chunks of bits.)
//!
//! This way, to search for an unoccupied dynamic allocation we iterate over
//! the set-bits vector which is 32 times more dense than our `free` list.
//! The additional density implies that we can query up to 8192 potential
//! dynamic storage allocations with a single contract storage look-up.

mod allocation;
mod allocator;
mod init;

#[cfg(test)]
mod tests;

pub use self::{
    allocation::DynamicAllocation,
    init::{
        initialize_for,
        ContractPhase,
    },
};
use self::{
    allocator::DynamicAllocator,
    init::on_call,
};

/// Returns a new dynamic storage allocation.
pub fn alloc() -> DynamicAllocation {
    on_call(DynamicAllocator::alloc)
}

/// Frees the given dynamic storage allocation.
///
/// This makes the given dynamic storage allocation available again
/// for new dynamic storage allocations.
pub fn free(allocation: DynamicAllocation) {
    on_call(|allocator| allocator.free(allocation))
}
