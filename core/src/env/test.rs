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

//! Public api to interact with the special testing environment.

use crate::env::{
    traits::EnvTypes,
    ContractEnv,
    ContractEnvStorage,
};

/// Returns the total number of reads to all storage entries.
pub fn total_reads() -> u64 {
    ContractEnvStorage::total_reads()
}

/// Returns the total number of writes to all storage entries.
pub fn total_writes() -> u64 {
    ContractEnvStorage::total_writes()
}

/// Sets the caller for the next calls to the given address.
pub fn set_caller<T: EnvTypes>(address: T::AccountId) {
    ContractEnv::<T>::set_caller(address)
}

/// Sets the timestamp for the next contract invocation.
pub fn set_now<T: EnvTypes>(timestamp: T::Moment) {
    ContractEnv::<T>::set_now(timestamp)
}

/// Sets the current block number for the next contract invocation.
pub fn set_block_number<T: EnvTypes>(block_number: T::BlockNumber) {
    ContractEnv::<T>::set_block_number(block_number)
}

/// Sets the contract balance for the next contract invocation.
pub fn set_balance<T: EnvTypes>(balance: T::Balance) {
    ContractEnv::<T>::set_balance(balance)
}

/// Returns an iterator over the uninterpreted bytes of all past emitted events.
pub fn emitted_events<T: EnvTypes>() -> impl Iterator<Item = Vec<u8>> {
    ContractEnv::<T>::emitted_events()
}
