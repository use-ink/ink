//! # Contract Caller
//!
//! Demonstrates calling into an `ink!` contract from a pallet.

#![cfg_attr(not(feature = "std"), no_std)]

ink_runtime::create_runtime!(ContractCallerRuntime, ContractCallerRuntimeInner, (), {
    ContractCaller: pallet_revive_caller,
});

impl pallet_revive_caller::Config for ContractCallerRuntimeInner {}
