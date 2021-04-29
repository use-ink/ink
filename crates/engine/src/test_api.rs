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
    ext::Engine,
    types::{
        AccountId,
        Balance,
    },
    AccountError,
    Error,
};
use std::collections::HashMap;

/// Record for an emitted event.
#[derive(Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<Vec<u8>>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct RecordedPrintlns {
    printlns: Vec<String>,
}

impl RecordedPrintlns {
    // Creates a new `Engine instance.
    pub fn new() -> Self {
        Self {
            printlns: Vec::new(),
        }
    }

    // Records a new println.
    pub fn record(&mut self, println: String) {
        self.printlns.push(println);
    }

    // Clears all recorded printlns.
    pub fn clear(&mut self) {
        self.printlns.clear();
    }
}

impl Default for RecordedPrintlns {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for RecordedPrintlns {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.printlns.into_iter()
    }
}

/// Recorder for relevant interactions with this crate.
pub struct DebugInfo {
    /// Emitted events recorder.
    emitted_events: Vec<EmittedEvent>,
    /// Emitted print messages recorder.
    emitted_printlns: RecordedPrintlns,
    /// The total number of reads to the storage.
    count_reads: HashMap<AccountId, usize>,
    /// The total number of writes to the storage.
    count_writes: HashMap<AccountId, usize>,
    /// The number of storage cells used by each account id.
    cells_per_account: HashMap<AccountId, HashMap<Vec<u8>, bool>>,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugInfo {
    // Creates a new `RecInstance instance.
    pub fn new() -> Self {
        Self {
            emitted_events: Vec::new(),
            emitted_printlns: RecordedPrintlns::new(),
            count_reads: HashMap::new(),
            count_writes: HashMap::new(),
            cells_per_account: HashMap::new(),
        }
    }

    /// Resets the recorder.
    pub fn reset(&mut self) {
        self.count_reads.clear();
        self.count_writes.clear();
        self.emitted_events.clear();
        self.emitted_printlns.clear();
        self.cells_per_account.clear();
    }

    /// Increases the number of storage writes for the supplied account by one.
    pub fn inc_writes(&mut self, account_id: AccountId) {
        self.count_writes
            .entry(account_id)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    /// Increases the number of storage reads for the supplied account by one.
    pub fn inc_reads(&mut self, account_id: AccountId) {
        self.count_reads
            .entry(account_id)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    /// Records that a cell exists for an account under `key`.
    pub fn record_cell_for_account(&mut self, account_id: AccountId, key: Vec<u8>) {
        self.cells_per_account
            .entry(account_id)
            .and_modify(|hm| {
                let _ = hm.insert(key.clone(), true);
            })
            .or_insert({
                let mut hm = HashMap::new();
                hm.insert(key, true);
                hm
            });
    }

    /// Removes the cell under `key` for the supplied account.
    pub fn remove_cell_for_account(&mut self, account_id: AccountId, key: Vec<u8>) {
        self.cells_per_account.entry(account_id).and_modify(|hm| {
            let _ = hm.remove(&key).expect("cell must exist for account");
        });
    }

    /// Records a println.
    pub fn record_println(&mut self, println: String) {
        self.emitted_printlns.record(println);
    }

    /// Records an event.
    pub fn record_event(&mut self, event: EmittedEvent) {
        self.emitted_events.push(event);
    }
}

impl Engine {
    /// Resets the environment.
    pub fn initialize_or_reset(&mut self) {
        self.exec_context.reset();
        self.storage.clear();
        self.debug_info.reset();
    }

    /// Returns the total number of reads and writes of the contract's storage.
    pub fn get_contract_storage_rw(&self, account_id: Vec<u8>) -> (usize, usize) {
        let account_id = AccountId::from(account_id);
        let reads = self.debug_info.count_reads.get(&account_id).unwrap_or(&0);
        let writes = self.debug_info.count_writes.get(&account_id).unwrap_or(&0);
        (*reads, *writes)
    }

    /// Sets a caller for the next call.
    pub fn set_caller(&mut self, output: Vec<u8>) {
        self.exec_context.caller = Some(output.into());
    }

    /// Sets the callee for the next call.
    pub fn set_callee(&mut self, output: Vec<u8>) {
        self.exec_context.callee = Some(output.into());
    }

    /// Returns the amount of storage cells used by the account `account_id`.
    ///
    /// Returns `None` if the `account_id` is non-existent.
    pub fn count_used_storage_cells(&self, account_id: &[u8]) -> Result<usize, Error> {
        let x = self
            .debug_info
            .cells_per_account
            .get(&account_id.to_owned().into())
            .ok_or_else(|| {
                Error::Account(AccountError::NoAccountForId(account_id.to_vec()))
            })?;
        Ok(x.len())
    }

    /// Returns the callee, i.e. the currently executing contract.
    pub fn get_callee(&self) -> Vec<u8> {
        self.exec_context.callee()
    }

    /// Returns the contents of the past performed environmental `println` in order.
    pub fn get_recorded_printlns(&self) -> RecordedPrintlns {
        self.debug_info.emitted_printlns.clone()
    }

    /// Returns the recorded emitted events in order.
    pub fn get_emitted_events(&self) -> impl Iterator<Item = EmittedEvent> {
        self.debug_info.emitted_events.clone().into_iter()
    }

    /// Returns the current balance of `account_id`.
    pub fn get_balance(&self, account_id: Vec<u8>) -> Result<Balance, Error> {
        self.storage
            .get_balance(&account_id)
            .ok_or(Error::Account(AccountError::NoAccountForId(account_id)))
    }

    /// Sets the balance of `account_id` to `new_balance`.
    pub fn set_balance(&mut self, account_id: Vec<u8>, new_balance: Balance) {
        self.storage.set_balance(&account_id, new_balance);
    }

    /// Sets the value transferred from the caller to the callee as part of the call.
    pub fn set_value_transferred(&mut self, value: Balance) {
        self.exec_context.value_transferred = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_getting_callee() {
        let mut engine = Engine::new();
        let account_id = vec![1; 32];
        engine.set_callee(account_id.clone());
        assert_eq!(engine.get_callee(), account_id);
    }

    #[test]
    fn count_cells_per_account_must_stay_the_same() {
        // given
        let mut engine = Engine::new();
        let account_id = vec![1; 32];
        engine.set_callee(account_id.clone());
        let key: &[u8; 32] = &[0x42; 32];
        engine.set_storage(key, &[0x05_u8; 5]);
        assert_eq!(engine.count_used_storage_cells(&account_id), Ok(1));

        // when
        // we set the storage a second time
        engine.set_storage(key, &[0x05_u8; 6]);

        // then
        // the amount of storage cells used must have stayed the same
        assert_eq!(engine.count_used_storage_cells(&account_id), Ok(1));
    }

    #[test]
    fn count_cells_per_account_must_be_reset() {
        // given
        let mut engine = Engine::new();
        let account_id = vec![1; 32];
        engine.set_callee(account_id.clone());
        let key: &[u8; 32] = &[0x42; 32];
        engine.set_storage(key, &[0x05_u8; 5]);
        assert_eq!(engine.count_used_storage_cells(&account_id), Ok(1));

        // when
        engine.clear_storage(key);

        // then
        assert_eq!(engine.count_used_storage_cells(&account_id), Ok(0));
    }
}
