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

pub mod db;
mod impls;
pub mod typed_encoded;
pub mod types;

use self::{
    db::{
        AccountError,
        AccountsDb,
        Account,
        Block,
        ChainSpec,
        CodeDb,
        ExecContext,
    },
    typed_encoded::{
        TypedEncoded,
        TypedEncodedError,
    },
    types::{
        OffAccountId,
        OffBalance,
        OffBlockNumber,
        OffHash,
        OffMoment,
    },
};
use super::OnInstance;
use core::cell::RefCell;
use derive_more::From;

#[derive(Debug, From)]
pub enum InstanceError {
    Account(AccountError),
    TypedEncoded(TypedEncodedError),
    #[from(ignore)]
    UninitializedBlocks,
    #[from(ignore)]
    UninitializedExecutionContext,
}

pub type Result<T> = core::result::Result<T, InstanceError>;

/// The off-chain environment.
///
/// Mainly used for off-chain testing.
pub struct EnvInstance {
    /// The accounts database of the environment.
    accounts: AccountsDb,
    /// Uploaded Wasm contract codes.
    codes: CodeDb,
    /// Current execution context and context.
    exec_context: Option<ExecContext>,
    /// The general chain spec.
    chain_spec: ChainSpec,
    /// The blocks of the chain.
    blocks: Vec<Block>,
}

impl EnvInstance {
    /// Creates a new uninitialized off-chain environment.
    pub fn uninitialized() -> Self {
        Self {
            accounts: AccountsDb::new(),
            codes: CodeDb::new(),
            exec_context: None,
            chain_spec: ChainSpec::uninitialized(),
            blocks: Vec::new(),
        }
    }

    /// Returns the current execution context.
    fn exec_context(&self) -> Result<&ExecContext> {
        self.exec_context.as_ref().ok_or(InstanceError::UninitializedExecutionContext)
    }

    /// Returns the current execution context.
    fn exec_context_mut(&mut self) -> Result<&mut ExecContext> {
        self.exec_context.as_mut().ok_or(InstanceError::UninitializedExecutionContext)
    }

    /// Returns the current block of the chain.
    fn current_block(&self) -> Result<&Block> {
        self.blocks.last().ok_or(InstanceError::UninitializedBlocks)
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
