// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    env2::{
        EnvTypes,
        test::{
            Storage,
        },
    },
};

/// The test environment.
///
/// This allows for limited off-chain testing of smart contracts
/// with enhanced support for introspection and mutation of the
/// emulated SRML contracts environment.
pub struct TestEnv<T>
where
    T: EnvTypes,
{
    /// The emulated contract storage.
    storage: Storage,
    /// The emulated chain state.
    state: ChainState<T>,
    /// The most current block.
    block: Block<T>,
    /// The current contract execution context.
    exec_context: ExecutionContext<T>,
}

impl<T> EnvTypes for TestEnv<T>
where
    T: EnvTypes,
{
    /// The type of an address.
    type AccountId = T::AccountId;
    /// The type of balances.
    type Balance = T::Balance;
    /// The type of hash.
    type Hash = T::Hash;
    /// The type of timestamps.
    type Moment = T::Moment;
    /// The type of block number.
    type BlockNumber = T::BlockNumber;
    /// The type of a call into the runtime
    type Call = T::Call;
}

/// The emulated chain state.
///
/// This stores general information about the chain.
pub struct ChainState<T>
where
    T: EnvTypes,
{
    /// The current gas price.
    gas_price: T::Balance,
}

/// A block within the emulated chain.
///
/// This stores information associated to blocks.
pub struct Block<T>
where
    T: EnvTypes,
{
    /// The number of the block.
    number: T::BlockNumber,
    /// The blocktime in milliseconds.
    now_in_ms: T::Moment,
}

/// An execution context is opened whenever a contract is being called or instantiated.
pub struct ExecutionContext<T>
where
    T: EnvTypes,
{
    /// The caller of the execution.
    caller: T::AccountId,
    /// The address of the called contract.
    callee: T::AccountId,
    /// The endowment for the call.
    endowment: T::Balance,
    /// The amount of gas left for further execution.
    gas_left: T::Balance,
    /// The limit of gas usage.
    ///
    /// There might be no limit thus `gas_left` is the actual limit then.
    gas_limit: Option<T::Balance>,
    /// The associated block for the execution.
    block: Block<T>,
}

/// Allocates new account IDs.
///
/// This is used whenever a new account or contract
/// is created on the emulated chain.
pub struct AccountIdAlloc<T>
where
    T: EnvTypes,
{
    /// The current account ID.
    current: T::AccountId,
}

impl<T> Default for AccountIdAlloc<T>
where
    T: EnvTypes,
    T::AccountId: From<usize>,
{
    fn default() -> Self {
        Self {
            current: 0.into(),
        }
    }
}

use core::ops::{
    AddAssign,
};

impl<T> AccountIdAlloc<T>
where
    T: EnvTypes,
    T::AccountId: AddAssign<usize> + Clone,
{
    pub fn next(&mut self) -> T::AccountId {
        self.current += 1;
        self.current.clone()
    }
}
