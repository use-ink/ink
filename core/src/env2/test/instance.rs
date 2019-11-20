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

use crate::{
    byte_utils,
    env2::{
        call::{
            CallData,
            Selector,
        },
        test::{
            storage::Storage,
            types::*,
            AccountsDb,
            AlreadyInitialized,
            Record,
            TypedEncoded,
        },
        EnvTypes,
    },
};

/// The instance of the test environment.
///
/// # Single Instance
///
/// This is basically the database of the actual test environment.
/// We need exactly one single instance of this type which the actual
/// `TestEnv` is going to access through `thread_local` storage.
/// Since `thread_local` storage doesn't allow for generics `TestEnvInstance`
/// needs to be `EnvTypes` agnostic.
///
/// # Type Safety
///
/// To counter the lost type safety of being `EnvTypes` agnostic
/// `TestEnvInstance` uses the `TypedEncoded` abstraction where possible
/// since it provides a small type-safe runtime-checked wrapper
/// arround the state.
///
/// # Default
///
/// The `thread_local` storage is using the `Default` implementation
/// of `TestEnvInstance` in order to initialize it thread locally.
/// However, since we are using `TypedEncoded` we need a separate initialization
/// routine to actually initialize those for their guarantees around type safe accesses.
/// To initialize `TestEnvInstance` type-safely `TestEnv` is using its `initialize_using`
/// routine which has certain constraints to the actual environmental types.
#[derive(Debug, Default)]
pub struct TestEnvInstance {
    /// The accounts registered on the chain.
    pub accounts: AccountsDb,
    /// The emulated chain state.
    pub state: ChainState,
    /// The current and latest block.
    pub block: Block,
    /// The current contract execution context.
    pub exec_context: ExecutionContext,
    /// Account ID generator.
    pub account_id_gen: AccountIdGen,
    /// Records of certain events and environmental interactions.
    pub records: Vec<Record>,
}

impl TestEnvInstance {
    /// Initializes `self` with a given encodable value.
    ///
    /// # Errors
    ///
    /// If `self` has already been initialized or is an initialized instance.
    pub fn try_initialize<E>(&mut self) -> Result<(), AlreadyInitialized>
    where
        E: EnvTypes,
        <E as EnvTypes>::AccountId: From<[u8; 32]>,
        <E as EnvTypes>::Balance: From<u128>,
        <E as EnvTypes>::BlockNumber: From<u64>,
        <E as EnvTypes>::Moment: From<u64>,
    {
        self.state.try_initialize::<E>(
            <E as EnvTypes>::Balance::from(1),
            <E as EnvTypes>::Balance::from(0),
        )?;
        self.block.try_initialize::<E>(
            <E as EnvTypes>::BlockNumber::from(0),
            <E as EnvTypes>::Moment::from(0),
        )?;
        self.exec_context.try_initialize::<E>(
            <E as EnvTypes>::AccountId::from([0x00; 32]),
            <E as EnvTypes>::AccountId::from([0x01; 32]),
            <E as EnvTypes>::Balance::from(1000),
            <E as EnvTypes>::Balance::from(500_000),
            None,
        )?;
        Ok(())
    }
}

/// The emulated chain state.
///
/// This stores general information about the chain.
#[derive(Debug, Clone, Default)]
pub struct ChainState {
    /// The emulated chain storage.
    pub storage: Storage,
    /// The current gas price.
    pub gas_price: Balance,
    /// The existential deposit.
    pub minimum_balance: Balance,
}

impl ChainState {
    /// Initializes `self` with a given encodable value.
    ///
    /// # Errors
    ///
    /// If `self` has already been initialized or is an initialized instance.
    pub fn try_initialize<E>(
        &mut self,
        gas_price: <E as EnvTypes>::Balance,
        minimum_balance: <E as EnvTypes>::Balance,
    ) -> Result<(), AlreadyInitialized>
    where
        E: EnvTypes,
        <E as EnvTypes>::Balance: From<u128>,
    {
        self.gas_price = TypedEncoded::from_origin(&gas_price);
        self.minimum_balance = TypedEncoded::from_origin(&minimum_balance);
        Ok(())
    }
}

/// A block within the emulated chain.
///
/// This stores information associated to blocks.
#[derive(Debug, Clone, Default)]
pub struct Block {
    /// The number of the block.
    pub number: BlockNumber,
    /// The blocktime in milliseconds.
    pub now_in_ms: Moment,
}

impl Block {
    /// Initializes `self` with a given encodable value.
    ///
    /// # Errors
    ///
    /// If `self` has already been initialized or is an initialized instance.
    pub fn try_initialize<E>(
        &mut self,
        number: <E as EnvTypes>::BlockNumber,
        now_in_ms: <E as EnvTypes>::Moment,
    ) -> Result<(), AlreadyInitialized>
    where
        E: EnvTypes,
    {
        self.number = TypedEncoded::from_origin(&number);
        self.now_in_ms = TypedEncoded::from_origin(&now_in_ms);
        Ok(())
    }
}

/// An execution context is opened whenever a contract is being called or instantiated.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The caller of the execution.
    pub caller: AccountId,
    /// The address of the called contract.
    pub callee: AccountId,
    /// The endowment for the call.
    pub transferred_balance: Balance,
    /// The amount of gas left for further execution.
    pub gas_left: Balance,
    /// The limit of gas usage.
    ///
    /// There might be no limit thus `gas_left` is the actual limit then.
    pub gas_limit: Option<Balance>,
    /// The raw call data for the contract execution.
    pub call_data: CallData,
    /// The output of the contract if any.
    ///
    /// Since this can be an arbitrary type we need to store it
    /// as its most general form: raw bytes.
    pub output: Option<Vec<u8>>,
}

impl ExecutionContext {
    /// Initializes `self` with a given encodable value.
    ///
    /// # Errors
    ///
    /// If `self` has already been initialized or is an initialized instance.
    pub fn try_initialize<E>(
        &mut self,
        caller: <E as EnvTypes>::AccountId,
        callee: <E as EnvTypes>::AccountId,
        transferred_balance: <E as EnvTypes>::Balance,
        gas_left: <E as EnvTypes>::Balance,
        gas_limit: Option<<E as EnvTypes>::Balance>,
    ) -> Result<(), AlreadyInitialized>
    where
        E: EnvTypes,
    {
        self.caller = TypedEncoded::from_origin(&caller);
        self.callee = TypedEncoded::from_origin(&callee);
        self.transferred_balance = TypedEncoded::from_origin(&transferred_balance);
        self.gas_left = TypedEncoded::from_origin(&gas_left);
        self.gas_limit = gas_limit.map(|gas_limit| TypedEncoded::from_origin(&gas_limit));
        Ok(())
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            caller: Default::default(),
            callee: Default::default(),
            transferred_balance: Default::default(),
            gas_left: Default::default(),
            gas_limit: Default::default(),
            call_data: CallData::new(Selector::from([0x00; 4])),
            output: None,
        }
    }
}

/// Allocates new account IDs.
///
/// This is used whenever a new account or contract
/// is created on the emulated chain.
#[derive(Debug, Clone, Default)]
pub struct AccountIdGen {
    /// The current account ID.
    current: [u8; 32],
}

impl AccountIdGen {
    /// Returns the next account ID.
    pub fn next<E>(&mut self) -> (AccountId, <E as EnvTypes>::AccountId)
    where
        E: EnvTypes,
        <E as EnvTypes>::AccountId: From<[u8; 32]>,
    {
        byte_utils::bytes_add_bytes(&mut self.current, &[0x01]);
        let account_id: <E as EnvTypes>::AccountId = self.current.into();
        let typed_encoded = TypedEncoded::from_origin(&account_id);
        (typed_encoded, account_id)
    }
}
