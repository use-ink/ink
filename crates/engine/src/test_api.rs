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

use crate::{
    ext::EnvInstance,
    types::{
        AccountId,
        Balance,
    },
    AccountError,
    Error,
    OnInstance,
};
use core::cell::RefCell;
use std::collections::HashMap;

pub struct RecInstance {
    /// Emitted events recorder.
    pub emitted_events: Vec<EmittedEvent>,
    /// Emitted print messages recorder.
    pub emitted_printlns: Vec<String>,
    /// The total number of reads to the storage.
    pub count_reads: HashMap<AccountId, usize>,
    /// The total number of writes to the storage.
    pub count_writes: HashMap<AccountId, usize>,
    /// The number of storage cells used by each account id.
    pub cells_per_account: HashMap<AccountId, usize>,
}

/// Record for an emitted event.
#[derive(Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<Vec<u8>>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

impl OnInstance for RecInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        thread_local!(
            static REC_INSTANCE: RefCell<RecInstance> = RefCell::new(RecInstance {
                emitted_events: Vec::new(),
                emitted_printlns: Vec::new(),
                count_reads: HashMap::new(),
                count_writes: HashMap::new(),
                cells_per_account: HashMap::new(),
            });
        );
        REC_INSTANCE.with(|instance| f(&mut instance.borrow_mut()))
    }
}

/// Resets the environment.
pub fn reset_environment() {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.reset();
        instance.balances.clear();
        instance.storage.clear();
    });
    <RecInstance as OnInstance>::on_instance(|instance| {
        instance.count_reads.clear();
        instance.count_writes.clear();
        instance.emitted_events.clear();
        instance.emitted_printlns.clear();
        instance.cells_per_account.clear();
    })
}

/// Returns the recorded emitted events in order.
pub fn get_emitted_events() -> Vec<EmittedEvent> {
    <RecInstance as OnInstance>::on_instance(|instance| instance.emitted_events.clone())
}

/// Returns the total number of reads and writes of the contract's storage.
pub fn get_contract_storage_rw(account_id: Vec<u8>) -> (usize, usize) {
    let account_id = AccountId::from(account_id);
    <RecInstance as OnInstance>::on_instance(|instance| {
        let reads = instance.count_reads.get(&account_id).unwrap_or(&0);
        let writes = instance.count_writes.get(&account_id).unwrap_or(&0);
        (*reads, *writes)
    })
}

/// Sets a caller for the next call.
pub fn set_caller(output: Vec<u8>) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.caller = output.into();
    });
}

/// Sets the callee for the next call.
pub fn set_callee(output: Vec<u8>) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.caller = output.into();
    });
}

/// Returns the amount of storage cells used by the account `account_id`.
///
/// Returns `None` if the `account_id` is non-existent.
pub fn count_used_storage_cells(account_id: Vec<u8>) -> Result<usize, Error> {
    <RecInstance as OnInstance>::on_instance(|instance| {
        instance
            .cells_per_account
            .get(&account_id.clone().into())
            .copied()
            .ok_or(Error::Account(AccountError::NoAccountForId(account_id)))
    })
}

/// Returns the callee, i.e. the currently executing contract.
pub fn get_callee() -> Vec<u8> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.callee.clone().into()
    })
}

/// Returns the contents of the past performed environmental `println` in order.
pub fn get_recorded_printlns() -> impl Iterator<Item = String> {
    <RecInstance as OnInstance>::on_instance(|instance| {
        instance.emitted_printlns.clone().into_iter()
    })
}

/// Sets the balance of `account_id` to `new_balance`.
pub fn set_balance(account_id: Vec<u8>, new_balance: Balance) {
    let account_id = AccountId::from(account_id);
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .balances
            .entry(account_id)
            .and_modify(|v| *v = new_balance)
            .or_insert(new_balance);
    });
}

/// Returns the current balance of `account_id`.
pub fn get_balance(account_id: Vec<u8>) -> Result<Balance, Error> {
    let acc = AccountId::from(account_id.clone());
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .balances
            .get(&acc)
            .copied()
            .ok_or(Error::Account(AccountError::NoAccountForId(account_id)))
    })
}

/// Sets the value transferred from the caller to the callee as part of the call.
pub fn set_value_transferred(value: Balance) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.value_transferred = value;
    });
}
