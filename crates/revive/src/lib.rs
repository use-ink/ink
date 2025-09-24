#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod evm;
mod hex_serde;
mod primitives;

pub use primitives::{
    CodeUploadResult,
    CodeUploadReturnValue,
    ContractResult,
    ExecReturnValue,
    InstantiateReturnValue,
    StorageDeposit,
};
