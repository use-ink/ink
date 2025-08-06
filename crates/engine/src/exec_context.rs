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

use super::types::{
    BlockNumber,
    BlockTimestamp,
};
use ink_primitives::{
    Address,
    U256,
};

/// The context of a contract execution.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Default)]
pub struct ExecContext {
    /// The caller of the contract execution. Might be user or another contract.
    ///
    /// TODO check next comment
    /// We don't know the specifics of the `AccountId` ‒ like how many bytes or what
    /// type of default `AccountId` makes sense ‒ they are left to be initialized
    /// by the crate which uses the `engine`. Methods which require a caller might
    /// panic when it has not been set.
    pub caller: Address,
    /// The callee of the contract execution. Might be user or another contract.
    ///
    /// We don't know the specifics of the `AccountId` ‒ like how many bytes or what
    /// type of default `AccountId` makes sense ‒ they are left to be initialized
    /// by the crate which uses the `engine`. Methods which require a callee might
    /// panic when it has not been set.
    pub callee: Option<Address>,
    /// The value transferred to the contract as part of the call.
    pub value_transferred: U256,
    /// The current block number.
    pub block_number: BlockNumber,
    /// The current block timestamp.
    pub block_timestamp: BlockTimestamp,
    /// Known contract accounts
    pub contracts: Vec<Address>,
}

impl ExecContext {
    /// Creates a new execution context.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the callee.
    pub fn callee(&self) -> Address {
        self.callee.expect("no callee has been set")
    }

    /// Resets the execution context
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    /// Set the block timestamp for the execution context.
    pub fn set_block_timestamp(&mut self, block_timestamp: BlockTimestamp) {
        self.block_timestamp = block_timestamp
    }

    /// Set the block number for the execution context.
    pub fn set_block_number(&mut self, block_number: BlockNumber) {
        self.block_number = block_number
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Address,
        ExecContext,
    };

    #[test]
    fn basic_operations() {
        let mut exec_cont = ExecContext::new();

        exec_cont.callee = Some(Address::from([13; 20]));
        exec_cont.caller = Address::from([14; 20]);
        exec_cont.value_transferred = 15.into();
        assert_eq!(exec_cont.callee(), Address::from([13; 20]));

        exec_cont.reset();

        let new_exec_cont = ExecContext::new();
        assert_eq!(exec_cont, new_exec_cont);
    }
}
