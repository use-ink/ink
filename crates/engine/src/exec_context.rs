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

use super::{
    accounts::AccountError,
    typed_encoded::TypedEncodedError,
    types::OffAccountId,
    Environment,
};

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

type Result<T> = core::result::Result<T, OffChainError>;

/// The context of a contract execution.
pub struct ExecContext {
    /// The caller of the contract execution.
    ///
    /// Might be user or another contract.
    pub caller: OffAccountId,
    /// The callee of the contract execution.
    pub callee: OffAccountId,
}

impl ExecContext {
    /// Returns the callee.
    pub fn callee<T>(&self) -> Result<T::AccountId>
    where
        T: Environment,
    {
        self.callee.decode().map_err(Into::into)
    }
}
