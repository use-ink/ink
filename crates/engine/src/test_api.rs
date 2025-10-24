// Copyright (C) Use Ink (UK) Ltd.
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
    AccountError,
    Error,
    ext::Engine,
    types::{
        Balance,
        BlockNumber,
        BlockTimestamp,
    },
};
use ink_primitives::{
    AccountId,
    Address,
    U256,
};
use std::collections::HashMap;

/// Record for an emitted event.
#[derive(Debug, Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<[u8; 32]>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

/// Recorder for relevant interactions with this crate.
pub struct DebugInfo {
    /// Emitted events recorder.
    emitted_events: Vec<EmittedEvent>,
    /// The total number of reads to the storage.
    count_reads: HashMap<Address, usize>,
    /// The total number of writes to the storage.
    count_writes: HashMap<Address, usize>,
    /// The number of storage cells used by each contract.
    cells_per_contract: HashMap<Address, HashMap<Vec<u8>, bool>>,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugInfo {
    // Creates a new `RecInstance` instance.
    pub fn new() -> Self {
        Self {
            emitted_events: Vec::new(),
            count_reads: HashMap::new(),
            count_writes: HashMap::new(),
            cells_per_contract: HashMap::new(),
        }
    }

    /// Resets the recorder.
    pub fn reset(&mut self) {
        self.count_reads.clear();
        self.count_writes.clear();
        self.emitted_events.clear();
        self.cells_per_contract.clear();
    }

    /// Increases the number of storage writes for the supplied account by one.
    #[allow(clippy::arithmetic_side_effects)] // todo
    pub fn inc_writes(&mut self, addr: Address) {
        self.count_writes
            .entry(addr)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    /// Increases the number of storage reads for the supplied account by one.
    #[allow(clippy::arithmetic_side_effects)] // todo
    pub fn inc_reads(&mut self, addr: Address) {
        self.count_reads
            .entry(addr)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    /// Records that a cell exists for an account under `key`.
    ///
    /// Calling this function multiple times won't change the fact that only
    /// one cell is recorded.
    pub fn record_cell_for_account(&mut self, addr: Address, key: Vec<u8>) {
        self.cells_per_contract
            .entry(addr)
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
    ///
    /// Returns the removed cell, if there was one.
    pub fn remove_cell_for_account(
        &mut self,
        addr: Address,
        key: Vec<u8>,
    ) -> Option<bool> {
        self.cells_per_contract
            .get_mut(&addr)
            .map(|hm| hm.remove(&key))
            .unwrap_or(None)
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
        self.database.clear();
        self.debug_info.reset();
    }

    /// Returns the total number of reads and writes of the contract's storage.
    pub fn get_contract_storage_rw(&self, addr: Address) -> (usize, usize) {
        let reads = self.debug_info.count_reads.get(&addr).unwrap_or(&0);
        let writes = self.debug_info.count_writes.get(&addr).unwrap_or(&0);
        (*reads, *writes)
    }

    /// Returns the total number of reads executed.
    pub fn count_reads(&self) -> usize {
        self.debug_info.count_reads.values().sum()
    }

    /// Returns the total number of writes executed.
    pub fn count_writes(&self) -> usize {
        self.debug_info.count_writes.values().sum()
    }

    /// Sets a caller for the next call.
    pub fn set_caller(&mut self, caller: Address) {
        self.exec_context.caller = caller;
    }

    /// Sets a known contract by adding it to a vector of known contracts accounts
    pub fn set_contract(&mut self, caller: Address) {
        self.exec_context.contracts.push(caller);
    }

    /// Sets the callee for the next call.
    pub fn set_callee(&mut self, callee: Address) {
        self.exec_context.callee = Some(callee);
    }

    /// Returns the amount of storage cells used by the contract `addr`.
    ///
    /// Returns `None` if the contract `addr` is non-existent.
    pub fn count_used_storage_cells(&self, addr: &Address) -> Result<usize, Error> {
        let cells = self
            .debug_info
            .cells_per_contract
            .get(addr)
            .ok_or(Error::Account(AccountError::NoContractForId(*addr)))?;
        Ok(cells.len())
    }

    /// Advances the chain by a single block.
    pub fn advance_block(&mut self) {
        self.exec_context.block_number = self
            .exec_context
            .block_number
            .checked_add(1)
            .expect("failed to add");
        self.exec_context.block_timestamp = self
            .exec_context
            .block_timestamp
            .checked_add(self.chain_spec.block_time)
            .expect("failed to add");
    }

    /// Returns the callee, i.e. the currently executing contract.
    pub fn get_callee(&self) -> Address {
        self.exec_context.callee()
    }

    /// Returns boolean value indicating whether the account is a contract
    pub fn is_contract(&self, addr: &Address) -> bool {
        self.exec_context.contracts.contains(addr)
    }

    /// Returns the recorded emitted events in order.
    pub fn get_emitted_events(&self) -> impl Iterator<Item = EmittedEvent> {
        self.debug_info.emitted_events.clone().into_iter()
    }

    /// Returns the current balance of `addr`.
    pub fn get_acc_balance(&self, addr: AccountId) -> Result<Balance, Error> {
        self.database
            .get_acc_balance(&addr)
            .ok_or(Error::Account(AccountError::NoAccountForId(addr)))
    }

    /// Sets the balance of `addr` to `new_balance`.
    pub fn set_acc_balance(&mut self, addr: AccountId, new_balance: Balance) {
        self.database.set_acc_balance(&addr, new_balance);
    }

    /// Returns the current balance of `addr`.
    pub fn get_balance(&self, addr: Address) -> Result<U256, Error> {
        self.database
            .get_balance(&addr)
            .ok_or(Error::Account(AccountError::NoContractForId(addr)))
    }

    /// Sets the balance of `addr` to `new_balance`.
    pub fn set_balance(&mut self, addr: Address, new_balance: U256) {
        self.database.set_balance(&addr, new_balance);
    }

    /// Sets the value transferred from the caller to the callee as part of the call.
    pub fn set_value_transferred(&mut self, value: U256) {
        self.exec_context.value_transferred = value;
    }

    /// Set the block timestamp for the execution context.
    pub fn set_block_timestamp(&mut self, new_block_timestamp: BlockTimestamp) {
        self.exec_context.block_timestamp = new_block_timestamp;
    }

    /// Set the block number for the execution context.
    pub fn set_block_number(&mut self, new_block_number: BlockNumber) {
        self.exec_context.block_number = new_block_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_getting_callee() {
        let mut engine = Engine::new();
        let addr = Address::from([1; 20]);
        engine.set_callee(addr);
        assert_eq!(engine.get_callee(), addr);
    }

    #[test]
    fn count_cells_per_account_must_stay_the_same() {
        // given
        let mut engine = Engine::new();
        let addr = Address::from([1; 20]);
        engine.set_callee(addr);
        let key: &[u8; 32] = &[0x42; 32];
        engine.set_storage(key, &[0x05_u8; 5]);
        assert_eq!(engine.count_used_storage_cells(&addr), Ok(1));

        // when
        // we set the storage a second time
        engine.set_storage(key, &[0x05_u8; 6]);

        // then
        // the amount of storage cells used must have stayed the same
        assert_eq!(engine.count_used_storage_cells(&addr), Ok(1));
    }

    #[test]
    fn count_cells_per_account_must_be_reset() {
        // given
        let mut engine = Engine::new();
        let addr = Address::from([1; 20]);
        engine.set_callee(addr);
        let key: &[u8; 32] = &[0x42; 32];
        engine.set_storage(key, &[0x05_u8; 5]);
        assert_eq!(engine.count_used_storage_cells(&addr), Ok(1));

        // when
        engine.clear_storage(key);

        // then
        assert_eq!(engine.count_used_storage_cells(&addr), Ok(0));
    }

    #[test]
    fn count_total_writes() {
        // given
        let mut engine = Engine::new();
        let key: &[u8; 32] = &[0x42; 32];

        // when
        engine.set_callee(Address::from([1; 20]));
        engine.set_storage(key, &[0x05_u8; 5]);
        engine.set_storage(key, &[0x05_u8; 6]);
        engine.get_storage(key).unwrap();

        engine.set_callee(Address::from([2; 20]));
        engine.set_storage(key, &[0x07_u8; 7]);
        engine.get_storage(key).unwrap();

        // then
        assert_eq!(engine.count_writes(), 3);
        assert_eq!(engine.count_reads(), 2);
    }
}
