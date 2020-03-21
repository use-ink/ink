// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

mod db;
mod hashing;
mod impls;
mod runtime_calls;
mod runtime_storage;
pub mod test_api;
mod typed_encoded;
mod types;

#[cfg(test)]
mod tests;

use self::{
    db::{
        Account,
        AccountsDb,
        Block,
        ChainSpec,
        Console,
        EmittedEvent,
        EmittedEventsRecorder,
        ExecContext,
    },
    runtime_calls::RuntimeCallHandler,
    runtime_storage::RuntimeStorage,
    typed_encoded::TypedEncoded,
    types::{
        OffAccountId,
        OffBalance,
        OffBlockNumber,
        OffCall,
        OffHash,
        OffTimestamp,
    },
};
pub use self::{
    db::{
        AccountError,
        PastPrints,
    },
    typed_encoded::TypedEncodedError,
};
use super::OnInstance;
use crate::env::EnvTypes;
use core::cell::RefCell;
use derive_more::From;

#[derive(Debug, From, PartialEq, Eq)]
pub enum OffChainError {
    Account(AccountError),
    TypedEncoded(TypedEncodedError),
    #[from(ignore)]
    UninitializedBlocks,
    #[from(ignore)]
    UninitializedExecutionContext,
    #[from(ignore)]
    UnregisteredRuntimeCallHandler,
}

pub type Result<T> = core::result::Result<T, OffChainError>;

/// The off-chain environment.
///
/// Mainly used for off-chain testing.
pub struct EnvInstance {
    /// The accounts database of the environment.
    accounts: AccountsDb,
    /// Current execution context and context.
    exec_context: Vec<ExecContext>,
    /// The general chain spec.
    chain_spec: ChainSpec,
    /// The blocks of the chain.
    blocks: Vec<Block>,
    /// The console to print debug contents.
    console: Console,
    /// The emulated runtime storage.
    runtime_storage: RuntimeStorage,
    /// The runtime calls handler.
    runtime_call_handler: RuntimeCallHandler,
    /// Emitted events recorder.
    emitted_events: EmittedEventsRecorder,
}

impl EnvInstance {
    /// Creates a new uninitialized off-chain environment.
    pub fn uninitialized() -> Self {
        Self {
            accounts: AccountsDb::new(),
            exec_context: Vec::new(),
            chain_spec: ChainSpec::uninitialized(),
            blocks: Vec::new(),
            console: Console::new(),
            runtime_storage: RuntimeStorage::new(),
            runtime_call_handler: RuntimeCallHandler::new(),
            emitted_events: EmittedEventsRecorder::new(),
        }
    }

    /// Initializes the whole off-chain environment.
    ///
    /// # Note
    ///
    /// This is needed since we operate on a static instance that cannot be
    /// made generic over all environmental types and thus we are required to
    /// initialize it upon program start uninitialized which is why we have
    /// `TypedEncoded` wrappers.
    ///
    /// The off-chain environment requires to be initialized before every usage.
    ///
    /// This routine implements a default initialization that should be fine
    /// for most use cases.
    pub fn initialize_as_default<T>(&mut self) -> crate::env::Result<()>
    where
        T: EnvTypes,
        <T as EnvTypes>::AccountId: From<[u8; 32]>,
    {
        use core::ops::Div as _;
        use num_traits::{
            Bounded as _,
            Zero as _,
        };
        let default_accounts = test_api::default_accounts::<T>()?;
        // Alice has half of the maximum possible amount.
        self.accounts.add_user_account::<T>(
            default_accounts.alice.clone(),
            T::Balance::max_value().div(T::Balance::from(2)),
        );
        // Bob has half the balance that alice got.
        self.accounts.add_user_account::<T>(
            default_accounts.bob,
            T::Balance::max_value().div(T::Balance::from(4)),
        );
        // All other default accounts have zero balance.
        self.accounts
            .add_user_account::<T>(default_accounts.charlie, T::Balance::zero());
        self.accounts
            .add_user_account::<T>(default_accounts.django, T::Balance::zero());
        self.accounts
            .add_user_account::<T>(default_accounts.eve, T::Balance::zero());
        self.accounts
            .add_user_account::<T>(default_accounts.frank, T::Balance::zero());
        // Initialize our first block.
        self.blocks.push(Block::new::<T>(
            T::BlockNumber::from(0),
            T::Timestamp::from(0),
        ));
        // Initialize chain specification.
        self.chain_spec.initialize_as_default::<T>()?;
        // Initialize the called contract account.
        let contract_account_id = T::AccountId::from([0x07; 32]);
        self.accounts.add_contract_account::<T>(
            contract_account_id.clone(),
            T::Balance::from(0),
            T::Balance::from(20),
        );
        // Initialize the execution context for the first contract execution.
        use crate::env::call::{
            CallData,
            Selector,
        };
        self.exec_context.push(
            ExecContext::build::<T>()
                .caller(default_accounts.alice)
                .callee(contract_account_id)
                .gas(T::Balance::from(500_000))
                .transferred_value(T::Balance::from(500))
                .call_data(CallData::new(Selector::from_str("call")))
                .finish(),
        );
        Ok(())
    }

    /// Advances the chain by a single block.
    pub fn advance_block<T>(&mut self) -> crate::env::Result<()>
    where
        T: EnvTypes,
    {
        let new_block_number = T::BlockNumber::from(self.blocks.len() as u32);
        let new_timestamp = self.current_block()?.timestamp::<T>()?
            + self.chain_spec.block_time::<T>()?;
        self.blocks
            .push(Block::new::<T>(new_block_number, new_timestamp));
        Ok(())
    }

    /// Returns the current execution context.
    fn exec_context(&self) -> Result<&ExecContext> {
        self.exec_context
            .last()
            .ok_or(OffChainError::UninitializedExecutionContext)
    }

    /// Returns the current execution context.
    fn exec_context_mut(&mut self) -> Result<&mut ExecContext> {
        self.exec_context
            .last_mut()
            .ok_or(OffChainError::UninitializedExecutionContext)
    }

    /// Returns the current block of the chain.
    fn current_block(&self) -> Result<&Block> {
        self.blocks.last().ok_or(OffChainError::UninitializedBlocks)
    }

    /// Returns a mutable reference to the current block of the chain.
    fn current_block_mut(&mut self) -> Result<&mut Block> {
        self.blocks
            .last_mut()
            .ok_or_else(|| OffChainError::UninitializedBlocks)
    }
}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        thread_local!(
            static INSTANCE: RefCell<EnvInstance> = RefCell::new(
                EnvInstance::uninitialized()
            )
        );
        INSTANCE.with(|instance| f(&mut instance.borrow_mut()))
    }
}
