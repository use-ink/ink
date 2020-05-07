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
//! there are probably never going to happen collissions anywhere at any time
//! if keys are chosen randomly. Using the built-in crypto hashers on unique
//! input we can be sure that there are never going to be collissions in this
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
//! This might seem a lot but given that there might be potentially tousands or
//! ten tousands of dynamic allocations at any given time this might not scale
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

#[cfg(test)]
mod tests;

pub use self::allocation::DynamicAllocation;
use self::allocator::DynamicAllocator;
use crate::storage2::traits2::pull_spread_root;
use cfg_if::cfg_if;
use ink_primitives::Key;

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

/// Resets the allocator to the initial values.
///
/// # Note
///
/// This function is only needed for testing when tools such as `miri` run all
/// tests on the same thread in sequence so every test has to reset the
/// statically allocated instances like the storage allocator before running.
#[cfg(test)]
pub fn reset_allocator() {
    assert_eq!(get_contract_phase(), ContractPhase::Deploy);
    on_call(|allocator| allocator.reset())
}

/// The default dynamic allocator key offset.
///
/// This is where the dynamic allocator is stored on the contract storage.
const DYNAMIC_ALLOCATOR_KEY_OFFSET: [u8; 32] = [0xFE; 32];

mod phase {
    use super::cfg_if;

    /// The phase in which a contract execution can be.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum ContractPhase {
        /// A contract has been deployed initially.
        Deploy,
        /// An already deployed contract has been called.
        Call,
    }

    cfg_if! {
        if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {

            /// The global static phase capturing the state.
            static mut PHASE: Option<ContractPhase> = None;

            /// Sets the contract phase (either call or deploy).
            ///
            /// # Note
            ///
            /// This must happen before the first dynamic storage allocation and thus
            /// generally as early as possible upon contract execution.
            ///
            /// # Panics
            ///
            /// If a contract phase has already been submitted.
            pub fn set_contract_phase(phase: ContractPhase) {
                assert!(unsafe { &PHASE }.is_none());
                unsafe { PHASE = Some(phase) };
            }

            /// Returns the contract phase.
            ///
            /// # Panics
            ///
            /// If no contract phase has yet been submitted.
            pub fn get_contract_phase() -> ContractPhase {
                unsafe { PHASE }.expect("a contract phase has not yet been submitted")
            }

        } else if #[cfg(feature = "std")] {

            use ::core::cell::Cell;
            thread_local!(
                static PHASE: Cell<Option<ContractPhase>> = Cell::new(None);
            );

            /// Sets the contract phase (either call or deploy).
            ///
            /// # Note
            ///
            /// This must happen before the first dynamic storage allocation and thus
            /// generally as early as possible upon contract execution.
            ///
            /// # Panics
            ///
            /// If a contract phase has already been submitted.
            pub fn set_contract_phase(new_phase: ContractPhase) {
                PHASE.with(|phase| {
                    phase.replace(Some(new_phase));
                });
            }

            /// Returns the contract phase.
            ///
            /// # Panics
            ///
            /// If no contract phase has yet been submitted.
            pub fn get_contract_phase() -> ContractPhase {
                PHASE.with(|phase| {
                    assert!(phase.get().is_some());
                    phase.get().expect("a contract phase has not yet been submitted")
                })
            }

        } else {
            compile_error! {
                "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
            }
        }
    }
}
use phase::get_contract_phase;
pub use phase::{
    set_contract_phase,
    ContractPhase,
};

cfg_if! {
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {

        // Runs the given closure with the dynamic allocator for contract calls.
        fn on_call<F, R>(f: F) -> R
        where
            F: FnOnce(&mut DynamicAllocator) -> R,
        {
            static mut INSTANCE: Option<DynamicAllocator> = None;
            // Lazily initialize the dynamic allocator if not done, yet.
            if unsafe { &INSTANCE }.is_none() {
                match get_contract_phase() {
                    ContractPhase::Deploy => unsafe {
                        INSTANCE = Some(
                            DynamicAllocator::new()
                        );
                    }
                    ContractPhase::Call => unsafe {
                        INSTANCE = Some(
                            pull_spread_root::<DynamicAllocator>(&Key(DYNAMIC_ALLOCATOR_KEY_OFFSET))
                        );
                    }
                }
            }
            f(unsafe { INSTANCE.as_mut().expect("uninitialized dynamic storage allocator") })
        }

    } else if #[cfg(feature = "std")] {

        // Off-chain environment:
        fn on_call<F, R>(f: F) -> R
        where
            F: FnOnce(&mut DynamicAllocator) -> R,
        {
            use ::core::cell::RefCell;
            thread_local!(
                static INSTANCE: RefCell<Option<DynamicAllocator>> = RefCell::new(None);
            );
            // Lazily initialize the dynamic allocator if not done, yet.
            INSTANCE.with(|instance| {
                if instance.borrow().is_none() {
                    match get_contract_phase() {
                        ContractPhase::Deploy => {
                            instance.replace_with(|_| Some(
                                DynamicAllocator::new()
                            ));
                        }
                        ContractPhase::Call => {
                            instance.replace_with(|_| Some(
                                pull_spread_root::<DynamicAllocator>(&Key(DYNAMIC_ALLOCATOR_KEY_OFFSET))
                            ));
                        }
                    }
                }
            });
            INSTANCE.with(|instance| f(&mut instance.borrow_mut().as_mut().expect("uninitialized dynamic storage allocator")))
        }

    } else {
        compile_error! {
            "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
        }
    }
}
