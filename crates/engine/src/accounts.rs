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

use crate::test_api::Error;

use super::{
    exec_context::OffChainError,
    typed_encoded::TypedEncodedError,
    types::OffAccountId,
};

use derive_more::From;

/// Errors encountered upon interacting with the accounts database.
#[derive(Debug, From, PartialEq, Eq)]
pub enum AccountError {
    TypedEncoded(TypedEncodedError),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(OffAccountId),
}

impl From<AccountError> for Error {
    fn from(account_error: AccountError) -> Self {
        Error::OffChain(OffChainError::Account(account_error))
    }
}

impl From<scale::Error> for AccountError {
    fn from(err: scale::Error) -> Self {
        AccountError::TypedEncoded(err.into())
    }
}
