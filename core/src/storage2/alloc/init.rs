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

use super::DynamicAllocator;
use crate::storage2::traits::pull_spread_root;
use cfg_if::cfg_if;
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
    /// The global instance has already been initialized successfully.
    Initialized(DynamicAllocator),
}

impl From<ContractPhase> for DynamicAllocatorState {
    fn from(phase: ContractPhase) -> Self {
        match phase {
            ContractPhase::Deploy => DynamicAllocatorState::UninitDeploy,
            ContractPhase::Call => DynamicAllocatorState::UninitCall,
        }
    }
}

cfg_if! {
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {
        // Procedures for the Wasm compilation:

        /// The global instance for the dynamic storage allocator.
        static mut GLOBAL_INSTANCE: DynamicAllocatorState = DynamicAllocatorState::UninitDeploy;

        /// Commands the (re-)initialization of the global instance for the dynamic
        /// storage allocator.
        pub fn initialize_for(phase: ContractPhase) {
            let instance = unsafe { &mut GLOBAL_INSTANCE };
            // We do not allow reinitialization for Wasm targets for performance reasons.
            if let DynamicAllocatorState::Initialized(_) = instance {
                panic!("cannot reinitialize dynamic storage allocator instance in Wasm");
            }
            *instance = phase.into();
        }

        /// Runs the given closure on the global instance for the dynamic storage allocator.
        pub fn on_call<F, R>(f: F) -> R
        where
            F: FnOnce(&mut DynamicAllocator) -> R,
        {
            let instance = unsafe { &mut GLOBAL_INSTANCE };
            match instance {
                DynamicAllocatorState::UninitDeploy => {
                    let mut allocator = DynamicAllocator::default();
                    let result = f(&mut allocator);
                    *instance = DynamicAllocatorState::Initialized(allocator);
                    result
                }
                DynamicAllocatorState::UninitCall => {
                    let mut allocator = pull_spread_root::<DynamicAllocator>(&Key(DYNAMIC_ALLOCATOR_KEY_OFFSET));
                    let result = f(&mut allocator);
                    *instance = DynamicAllocatorState::Initialized(allocator);
                    result
                }
                DynamicAllocatorState::Initialized(ref mut allocator) => {
                    f(allocator)
                }
            }
        }

    } else if #[cfg(feature = "std")] {
        // Procedures for the off-chain environment and testing compilation:

        use ::core::cell::RefCell;
        thread_local!(
            /// The global instance for the dynamic storage allocator.
            static GLOBAL_INSTANCE: RefCell<DynamicAllocatorState> = RefCell::new(DynamicAllocatorState::UninitDeploy);
        );

        /// Commands the (re-)initialization of the global instance for the dynamic
        /// storage allocator.
        pub fn initialize_for(phase: ContractPhase) {
            GLOBAL_INSTANCE.with(|instance| {
                instance.replace_with(|_| phase.into())
            });
        }

        /// Runs the given closure on the global instance for the dynamic storage allocator.
        pub fn on_call<F, R>(f: F) -> R
        where
            F: FnOnce(&mut DynamicAllocator) -> R,
        {
            GLOBAL_INSTANCE.with(|instance| {
                match &mut *instance.borrow_mut() {
                    instance @ DynamicAllocatorState::UninitDeploy => {
                        let mut allocator = DynamicAllocator::default();
                        let result = f(&mut allocator);
                        *instance = DynamicAllocatorState::Initialized(allocator);
                        result
                    }
                    instance @ DynamicAllocatorState::UninitCall => {
                        let mut allocator = pull_spread_root::<DynamicAllocator>(&Key(DYNAMIC_ALLOCATOR_KEY_OFFSET));
                        let result = f(&mut allocator);
                        *instance = DynamicAllocatorState::Initialized(allocator);
                        result
                    }
                    DynamicAllocatorState::Initialized(instance) => {
                        f(instance)
                    }
                }
            })
        }

    } else {
        compile_error! {
            "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
        }
    }
}
