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

mod db;
mod impls;
mod runtime_calls;
mod runtime_storage;
pub mod test_api;
mod typed_encoded;
mod types;

use self::{
    db::{
        Account,
        AccountsDb,
        Block,
        ChainSpec,
        CodeDb,
        Console,
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
        OffMoment,
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
use core::cell::RefCell;
use derive_more::From;

#[derive(Debug, From)]
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
    /// Uploaded Wasm contract codes.
    #[allow(
        dead_code,
        // Needed as soon as we support to execute contracts
        // directly through the off-chain environment.
    )]
    codes: CodeDb,
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
}

impl EnvInstance {
    /// Creates a new uninitialized off-chain environment.
    pub fn uninitialized() -> Self {
        Self {
            accounts: AccountsDb::new(),
            codes: CodeDb::new(),
            exec_context: Vec::new(),
            chain_spec: ChainSpec::uninitialized(),
            blocks: Vec::new(),
            console: Console::new(),
            runtime_storage: RuntimeStorage::new(),
            runtime_call_handler: RuntimeCallHandler::new(),
        }
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
        self.blocks.last_mut().ok_or(OffChainError::UninitializedBlocks)
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
