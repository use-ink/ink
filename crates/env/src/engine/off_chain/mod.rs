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

mod call_data;
mod chain_extension;
mod db;
mod hashing;
mod impls;
pub mod test_api;
mod typed_encoded;
mod types;

#[cfg(test)]
mod tests;

pub use self::{
    call_data::CallData,
    db::{
        AccountError,
        DebugMessages,
        EmittedEvent,
    },
    typed_encoded::TypedEncodedError,
};
use self::{
    chain_extension::ChainExtensionHandler,
    db::{
        Account,
        AccountsDb,
        Block,
        ChainSpec,
        DebugBuffer,
        EmittedEventsRecorder,
        ExecContext,
    },
    typed_encoded::TypedEncoded,
    types::{
        OffAccountId,
        OffBalance,
        OffBlockNumber,
        OffHash,
        OffTimestamp,
    },
};
use super::OnInstance;
use crate::Environment;
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
    UnregisteredChainExtension,
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
    /// The debug buffer to collect debug messages and print them to stdout.
    debug_buf: DebugBuffer,
    /// Handler for registered chain extensions.
    chain_extension_handler: ChainExtensionHandler,
    /// Emitted events recorder.
    emitted_events: EmittedEventsRecorder,
    /// Set to true to disable clearing storage
    clear_storage_disabled: bool,
}

impl EnvInstance {
    /// Creates a new uninitialized off-chain environment.
    pub fn uninitialized() -> Self {
        Self {
            accounts: AccountsDb::new(),
            exec_context: Vec::new(),
            chain_spec: ChainSpec::uninitialized(),
            blocks: Vec::new(),
            debug_buf: DebugBuffer::new(),
            chain_extension_handler: ChainExtensionHandler::new(),
            emitted_events: EmittedEventsRecorder::new(),
            clear_storage_disabled: false,
        }
    }

    /// Returns `true` if the off-chain environment is uninitialized.
    pub fn is_initialized(&self) -> bool {
        !self.exec_context.is_empty()
    }

    /// Either resets or initializes the off-chain environment to default values.
    pub fn initialize_or_reset_as_default<T>(&mut self) -> crate::Result<()>
    where
        T: Environment,
        <T as Environment>::AccountId: From<[u8; 32]>,
    {
        if self.is_initialized() {
            self.reset()
        }
        self.initialize_as_default::<T>()?;
        Ok(())
    }

    /// Resets the off-chain environment to uninitialized state.
    pub fn reset(&mut self) {
        self.accounts.reset();
        self.exec_context.clear();
        self.chain_spec.reset();
        self.blocks.clear();
        self.debug_buf.reset();
        self.chain_extension_handler.reset();
        self.emitted_events.reset();
        self.clear_storage_disabled = false;
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
    pub fn initialize_as_default<T>(&mut self) -> crate::Result<()>
    where
        T: Environment,
        <T as Environment>::AccountId: From<[u8; 32]>,
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
            T::Balance::max_value().div(T::Balance::from(2u32)),
        );
        // Bob has half the balance that alice got.
        self.accounts.add_user_account::<T>(
            default_accounts.bob,
            T::Balance::max_value().div(T::Balance::from(4u32)),
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
            T::BlockNumber::from(0u32),
            T::Timestamp::from(0u32),
        ));
        // Initialize chain specification.
        self.chain_spec.initialize_as_default::<T>()?;
        // Initialize the called contract account.
        let contract_account_id = T::AccountId::from([0x07; 32]);
        self.accounts.add_contract_account::<T>(
            contract_account_id.clone(),
            T::Balance::from(0u32),
            T::Balance::from(20u32),
        );
        // Initialize the execution context for the first contract execution.
        use crate::call::Selector;
        // The below selector bytes are incorrect but since calling does not work
        // yet we do not have to fix this now.
        let selector_bytes_for_call = [0x00; 4];
        self.exec_context.push(
            ExecContext::build::<T>()
                .caller(default_accounts.alice)
                .callee(contract_account_id)
                .gas(500_000u64)
                .transferred_value(T::Balance::from(500u32))
                .call_data(CallData::new(Selector::new(selector_bytes_for_call)))
                .finish(),
        );
        Ok(())
    }

    /// Advances the chain by a single block.
    pub fn advance_block<T>(&mut self) -> crate::Result<()>
    where
        T: Environment,
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
            .ok_or(OffChainError::UninitializedBlocks)
    }

    fn chain_spec_mut(&mut self) -> &mut ChainSpec {
        &mut self.chain_spec
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
