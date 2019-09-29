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
    byte_utils,
    env2::{
        DefaultSrmlTypes,
        call::{
            Selector,
            CallData,
        },
        property,
        test::Storage,
        utils::{
            EnlargeTo,
            Reset,
        },
        EnvTypes,
        GetProperty,
        SetProperty,
        types,
    },
};
use core::marker::PhantomData;

type DefaultAccountId = types::AccountId;
type DefaultBalance = types::Balance;
type DefaultHash = types::Hash;
type DefaultMoment = types::Moment;
type DefaultBlockNumber = types::BlockNumber;

/// The instance of the test environment.
///
/// This allows for limited off-chain testing of smart contracts
/// with enhanced support for introspection and mutation of the
/// emulated SRML contracts environment.
pub struct TestEnvInstance<T>
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

impl<T> Default for TestEnvInstance<T>
where
    T: EnvTypes,
    ChainState<T>: Default,
    Block<T>: Default,
    ExecutionContext<T>: Default,
{
    fn default() -> Self {
        Self {
            storage: Default::default(),
            state: Default::default(),
            block: Default::default(),
            exec_context: Default::default(),
        }
    }
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
#[derive(Debug, Clone)]
pub struct ChainState<T>
where
    T: EnvTypes,
{
    /// The current gas price.
    gas_price: T::Balance,
}

impl<T> Default for ChainState<T>
where
    T: EnvTypes,
    <T as EnvTypes>::Balance: From<DefaultBalance>,
{
    fn default() -> Self {
        Self {
            gas_price: 0.into(),
        }
    }
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

impl<T> Default for Block<T>
where
    T: EnvTypes,
    <T as EnvTypes>::BlockNumber: From<DefaultBlockNumber>,
    <T as EnvTypes>::Moment: From<DefaultMoment>,
{
    fn default() -> Self {
        Self {
            number: 0.into(),
            now_in_ms: 0.into(),
        }
    }
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
    /// The raw call data for the contract execution.
    call_data: CallData,
    /// The limit of gas usage.
    ///
    /// There might be no limit thus `gas_left` is the actual limit then.
    gas_limit: Option<T::Balance>,
    /// The associated block for the execution.
    block: Block<T>,
}

impl<T> Default for ExecutionContext<T>
where
    T: EnvTypes,
    <T as EnvTypes>::AccountId: From<DefaultAccountId>,
    <T as EnvTypes>::Balance: From<DefaultBalance>,
    Block<T>: Default,
{
    fn default() -> Self {
        Self {
            caller: DefaultAccountId::from([0x00; 32]).into(),
            callee: DefaultAccountId::from([0x01; 32]).into(),
            endowment: 0.into(),
            gas_left: 0.into(),
            call_data: CallData::new(Selector::from([0, 1, 2, 3])),
            gas_limit: None,
            block: Default::default(),
        }
    }
}

/// Allocates new account IDs.
///
/// This is used whenever a new account or contract
/// is created on the emulated chain.
pub struct AccountIdAlloc<T> {
    /// The current account ID.
    current: [u8; 32],
    /// Environmental types marker.
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for AccountIdAlloc<T> {
    fn default() -> Self {
        Self {
            current: [0x0; 32],
            marker: Default::default(),
        }
    }
}

impl<T> AccountIdAlloc<T>
where
    T: EnvTypes,
    T::AccountId: From<[u8; 32]>,
{
    pub fn next(&mut self) -> T::AccountId {
        byte_utils::bytes_add_bytes(&mut self.current, &[0x01]);
        self.current.into()
    }
}

pub struct TestEnv<T> {
    marker: PhantomData<fn() -> T>,
}

impl<T> GetProperty<property::Input<Self>> for TestEnv<T>
where
    T: EnvTypes,
{
    fn get_property<I>(
        buffer: &mut I,
    ) -> <property::Input<Self> as property::ReadProperty>::In
    where
        I: AsMut<[u8]> + EnlargeTo,
    {
        // self.exec_context.call_data.clone()
        unimplemented!()
    }
}
