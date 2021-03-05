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
    ext::ENV_INSTANCE,
    types::AccountId,
    AccountError,
};
use core::cell::RefCell;

pub struct RecInstance {
    /// Emitted events recorder. Only required for tests.
    pub emitted_events: Vec<EmittedEvent>,
    /// The total number of reads to the storage.
    pub count_reads: usize,
    /// The total number of writes to the storage.
    pub count_writes: usize,
}

/// Record for an emitted event.
#[derive(Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<Vec<u8>>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

thread_local!(
    pub static REC_INSTANCE: RefCell<RecInstance> = RefCell::new(RecInstance {
        emitted_events: Vec::new(),
        count_reads: 0,
        count_writes: 0,
    });
);

/// Resets the recorder.
pub fn reset() {
    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.count_reads = 0;
        instance.count_writes = 0;
        instance.emitted_events = Vec::new();
    })
}

/// Returns the recorded emitted events in order.
pub fn get_emitted_events() -> Vec<EmittedEvent> {
    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.emitted_events.clone()
    })
}

/// Returns the total number of reads and writes of the contract's storage.
pub fn get_contract_storage_rw(
    _account_id: AccountId,
) -> Result<(usize, usize), AccountError> {
    // TODO `_account_id` is not considered yet!
    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        Ok((instance.count_reads, instance.count_writes))
    })
}

pub fn set_caller(output: Vec<u8>) {
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance
            .exec_context
            .as_mut()
            .expect("uninitialized context")
            .caller = output.into();
    });
}

pub fn count_used_storage_cells() -> usize {
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.storage.len()
    })
}

/// Returns the account id of the currently executing contract.
pub fn get_current_contract_account_id() -> Vec<u8> {
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow();
        let cont = instance.exec_context.as_ref().expect("no exec context");
        cont.callee.clone().into()
    })
}
