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

use super::DynamicAllocator;
use crate::traits::{
    pull_spread_root,
    push_spread_root,
};
use cfg_if::cfg_if;
use core::{
    mem,
    mem::ManuallyDrop,
};
use ink_primitives::Key;

/// The default dynamic allocator key offset.
///
/// This is where the dynamic allocator is stored on the contract storage.
const DYNAMIC_ALLOCATOR_KEY_OFFSET: [u8; 32] = [0xFE; 32];

/// The phase in which a contract execution can be.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContractPhase {
    /// Initializes the global dynamic storage allocator from scratch.
    ///
    /// Upon initialization it will be created from scratch as if the
    /// contract has been deployed for the first time.
    Deploy,
    /// Initializes the global dynamic storage allocator from storage.
    ///
    /// Upon initialization the dynamic storage allocator will be pulled
    /// from the contract storage with the assumption that a former
    /// contract deployment has already taken place in the past.
    Call,
}

/// The state of the dynamic allocator global instance.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum DynamicAllocatorState {
    /// The global instance has not yet been initialized.
    ///
    /// Upon initialization it will be created from scratch as if the
    /// contract has been deployed for the first time.
    UninitDeploy,
    /// The global instance has not yet been initialized.
    ///
    /// Upon initialization it will be pulled from the contract storage
    /// with the assumption that a former contract deployment has already
    /// taken place in the past.
    UninitCall,
    /// The global instance has been initialized successfully and can be used.
    Initialized(DynamicAllocator),
    /// The global instance has been finalized and can no longer be used.
    Finalized,
}

impl From<ContractPhase> for DynamicAllocatorState {
    fn from(phase: ContractPhase) -> Self {
        match phase {
            ContractPhase::Deploy => DynamicAllocatorState::UninitDeploy,
            ContractPhase::Call => DynamicAllocatorState::UninitCall,
        }
    }
}

impl DynamicAllocatorState {
    /// Initializes the global dynamic storage allocator instance.
    ///
    /// The `phase` parameter describes for which execution phase the dynamic
    /// storage allocator needs to be initialized since this is different
    /// in contract instantiations and calls.
    pub fn initialize(&mut self, phase: ContractPhase) {
        match self {
            DynamicAllocatorState::Initialized(_)
                // We only perform this check on Wasm compilation to avoid
                // some overly constrained check for the off-chain testing.
                if cfg!(all(not(feature = "std"), target_arch = "wasm32")) =>
            {
                panic!(
                    "cannot initialize the dynamic storage \
                     allocator instance twice in Wasm",
                )
            }
            DynamicAllocatorState::Finalized => {
                panic!(
                    "cannot initialize the dynamic storage \
                 allocator after it has been finalized",
                )
            }
            state => {
                *state = phase.into();
            }
        }
    }

    /// Finalizes the global instance for the dynamic storage allocator.
    ///
    /// The global dynamic storage allocator must not be used after this!
    pub fn finalize(&mut self) {
        match self {
            DynamicAllocatorState::Initialized(allocator) => {
                // Push all state of the global dynamic storage allocator
                // instance back onto the contract storage.
                push_spread_root::<DynamicAllocator>(
                    &allocator,
                    &Key::from(DYNAMIC_ALLOCATOR_KEY_OFFSET),
                );
                // Prevent calling `drop` on the dynamic storage allocator
                // instance since this would clear all contract storage
                // again.
                let _ = ManuallyDrop::new(mem::take(allocator));
                *self = DynamicAllocatorState::Finalized;
            }
            DynamicAllocatorState::Finalized => {
                panic!(
                    "cannot finalize the dynamic storage allocator \
                     after it has already been finalized"
                )
            }
            DynamicAllocatorState::UninitCall | DynamicAllocatorState::UninitDeploy => {
                // Nothing to do in these states.
            }
        }
    }

    /// Runs the closure on the global instance for the dynamic storage allocator.
    ///
    /// Will automatically initialize the global allocator instance if it has not
    /// yet been initialized.
    ///
    /// # Panics
    ///
    /// If the global dynamic storage allocator instance has already been finalized.
    pub fn on_instance<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut DynamicAllocator) -> R,
    {
        match self {
            DynamicAllocatorState::UninitDeploy => {
                let mut allocator = DynamicAllocator::default();
                let result = f(&mut allocator);
                *self = DynamicAllocatorState::Initialized(allocator);
                result
            }
            DynamicAllocatorState::UninitCall => {
                let mut allocator = pull_spread_root::<DynamicAllocator>(&Key::from(
                    DYNAMIC_ALLOCATOR_KEY_OFFSET,
                ));
                let result = f(&mut allocator);
                *self = DynamicAllocatorState::Initialized(allocator);
                result
            }
            DynamicAllocatorState::Initialized(ref mut allocator) => f(allocator),
            DynamicAllocatorState::Finalized => {
                panic!(
                    "cannot operate on the dynamic storage \
                     allocator after it has been finalized"
                );
            }
        }
    }
}

cfg_if! {
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {
        // Procedures for the Wasm compilation:

        /// The global instance for the dynamic storage allocator.
        static mut GLOBAL_INSTANCE: DynamicAllocatorState = DynamicAllocatorState::UninitDeploy;

        /// Forwards to the `initialize` of the global dynamic storage allocator instance.
        pub fn initialize(phase: ContractPhase) {
            // SAFETY: Accessing the global allocator in Wasm mode is single
            //         threaded and will not return back a reference to its
            //         internal state. Also the `initialize` method won't
            //         re-enter the dynamic storage in any possible way.
            unsafe { &mut GLOBAL_INSTANCE }.initialize(phase);
        }

        /// Forwards to the `finalize` of the global dynamic storage allocator instance.
        pub fn finalize() {
            // SAFETY: Accessing the global allocator in Wasm mode is single
            //         threaded and will not return back a reference to its
            //         internal state. Also the `finalize` method won't
            //         re-enter the dynamic storage in any possible way.
            unsafe { &mut GLOBAL_INSTANCE }.finalize();
        }

        /// Forwards to the `on_instance` of the global dynamic storage allocator instance.
        pub fn on_instance<F, R>(f: F) -> R
        where
            F: FnOnce(&mut DynamicAllocator) -> R,
        {
            // SAFETY: Accessing the global allocator in Wasm mode is single
            //         threaded and will not return back a reference to its
            //         internal state. Also this is an internal API only called
            //         through `alloc` and `free` both of which do not return
            //         anything that could allow to re-enter the dynamic storage
            //         allocator instance.
            unsafe { &mut GLOBAL_INSTANCE }.on_instance(f)
        }

    } else if #[cfg(feature = "std")] {
        // Procedures for the off-chain environment and testing compilation:

        use ::core::cell::RefCell;
        thread_local!(
            /// The global instance for the dynamic storage allocator.
            static GLOBAL_INSTANCE: RefCell<DynamicAllocatorState> = RefCell::new(
                DynamicAllocatorState::UninitDeploy
            );
        );
        /// Forwards to the `initialize` of the global dynamic storage allocator instance.
        pub fn initialize(phase: ContractPhase) {
            GLOBAL_INSTANCE.with(|instance| {
                instance.borrow_mut().initialize(phase)
            });
        }

        /// Forwards to the `finalize` of the global dynamic storage allocator instance.
        pub fn finalize() {
            GLOBAL_INSTANCE.with(|instance| {
                instance.borrow_mut().finalize()
            });
        }

        /// Forwards to the `on_instance` of the global dynamic storage allocator instance.
        pub fn on_instance<F, R>(f: F) -> R
        where
            F: FnOnce(&mut DynamicAllocator) -> R,
        {
            GLOBAL_INSTANCE.with(|instance| {
                instance.borrow_mut().on_instance(f)
            })
        }

    } else {
        compile_error! {
            "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
        }
    }
}
