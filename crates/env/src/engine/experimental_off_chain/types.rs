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

//! Contains the necessary conversions from `ink_engine` types to types
//! of this crate.

use super::{
    test_api::EmittedEvent,
    AccountError,
    Error,
    OffChainError,
};

impl From<ink_engine::test_api::EmittedEvent> for EmittedEvent {
    fn from(evt: ink_engine::test_api::EmittedEvent) -> Self {
        EmittedEvent {
            topics: evt.topics,
            data: evt.data,
        }
    }
}

impl From<ink_engine::Error> for Error {
    fn from(err: ink_engine::Error) -> Self {
        let e = match err {
            ink_engine::Error::Account(acc) => OffChainError::Account(acc.into()),
            ink_engine::Error::UninitializedBlocks => OffChainError::UninitializedBlocks,
            ink_engine::Error::UninitializedExecutionContext => {
                OffChainError::UninitializedExecutionContext
            }
            ink_engine::Error::UnregisteredChainExtension => {
                OffChainError::UnregisteredChainExtension
            }
        };
        Error::OffChain(e)
    }
}

impl From<ink_engine::AccountError> for AccountError {
    fn from(err: ink_engine::AccountError) -> Self {
        match err {
            ink_engine::AccountError::Decoding(e) => AccountError::Decoding(e),
            ink_engine::AccountError::UnexpectedUserAccount => {
                AccountError::UnexpectedUserAccount
            }
            ink_engine::AccountError::NoAccountForId(acc) => {
                AccountError::NoAccountForId(acc)
            }
        }
    }
}

impl From<ink_engine::AccountError> for Error {
    fn from(account_error: ink_engine::AccountError) -> Self {
        Error::OffChain(OffChainError::Account(account_error.into()))
    }
}
