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
        call::{
            CallData,
            Selector,
        },
        property,
        test::{
            Storage,
            TypedEncoded,
        },
        types,
        utils::{
            EnlargeTo,
            Reset,
        },
        DefaultSrmlTypes,
        EnvTypes,
        GetProperty,
        SetProperty,
    },
};
use core::{
    cell::RefCell,
    marker::PhantomData,
};

/// Type markers used in conjunction with `TypedEncoded`.
mod type_marker {
    /// Type marker representing an environmental `AccountId`.
    #[derive(Debug, Clone, Copy)]
    pub enum AccountId {}
    /// Type marker representing an environmental `Balance`.
    #[derive(Debug, Clone, Copy)]
    pub enum Balance {}
    /// Type marker representing an environmental `Hash`.
    #[derive(Debug, Clone, Copy)]
    pub enum Hash {}
    /// Type marker representing an environmental `Moment`.
    #[derive(Debug, Clone, Copy)]
    pub enum Moment {}
    /// Type marker representing an environmental `BlockNumber`.
    #[derive(Debug, Clone, Copy)]
    pub enum BlockNumber {}
    /// Type marker representing an environmental `Call`.
    #[derive(Debug, Clone, Copy)]
    pub enum Call {}
}

/// Environmental account ID type.
pub type AccountId = TypedEncoded<type_marker::AccountId>;
/// Environmental balance type.
pub type Balance = TypedEncoded<type_marker::Balance>;
/// Environmental hash type.
pub type Hash = TypedEncoded<type_marker::Hash>;
/// Environmental moment (block time) type.
pub type Moment = TypedEncoded<type_marker::Moment>;
/// Environmental block number type.
pub type BlockNumber = TypedEncoded<type_marker::BlockNumber>;
/// Environmental call (runtime dispatch) type.
pub type Call = TypedEncoded<type_marker::Call>;

/// The instance of the test environment.
///
/// This allows for limited off-chain testing of smart contracts
/// with enhanced support for introspection and mutation of the
/// emulated SRML contracts environment.
#[derive(Debug, Default)]
pub struct TestEnvInstance {
    /// The emulated contract storage.
    storage: Storage,
    /// The emulated chain state.
    state: ChainState,
    /// The most current block.
    block: Block,
    /// The current contract execution context.
    exec_context: ExecutionContext,
}

/// The emulated chain state.
///
/// This stores general information about the chain.
#[derive(Debug, Clone, Default)]
pub struct ChainState {
    /// The current gas price.
    gas_price: Balance,
}

/// A block within the emulated chain.
///
/// This stores information associated to blocks.
#[derive(Debug, Clone, Default)]
pub struct Block {
    /// The number of the block.
    number: BlockNumber,
    /// The blocktime in milliseconds.
    now_in_ms: Moment,
}

/// An execution context is opened whenever a contract is being called or instantiated.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The caller of the execution.
    caller: AccountId,
    /// The address of the called contract.
    callee: AccountId,
    /// The endowment for the call.
    endowment: Balance,
    /// The amount of gas left for further execution.
    gas_left: Balance,
    /// The raw call data for the contract execution.
    call_data: CallData,
    /// The limit of gas usage.
    ///
    /// There might be no limit thus `gas_left` is the actual limit then.
    gas_limit: Option<Balance>,
    /// The associated block for the execution.
    block: Block,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            caller: Default::default(),
            callee: Default::default(),
            endowment: Default::default(),
            gas_left: Default::default(),
            call_data: CallData::new(Selector::from([0x00; 4])),
            gas_limit: Default::default(),
            block: Default::default(),
        }
    }
}
