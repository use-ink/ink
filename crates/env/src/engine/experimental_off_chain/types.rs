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
    test_api::EmittedEvent,
    AccountError,
    Error,
    OffChainError,
};

// Conversion from `ink_engine` type to crate type
impl From<ink_engine::EmittedEvent> for EmittedEvent {
    fn from(evt: ink_engine::EmittedEvent) -> Self {
        EmittedEvent {
            topics: evt.topics,
            data: evt.data,
        }
    }
}

impl From<ink_engine::OffChainError> for Error {
    fn from(err: ink_engine::OffChainError) -> Self {
        let e = match err {
            ink_engine::OffChainError::Account(acc) => OffChainError::Account(acc.into()),
            ink_engine::OffChainError::UninitializedBlocks => {
                OffChainError::UninitializedBlocks
            }
            ink_engine::OffChainError::UninitializedExecutionContext => {
                OffChainError::UninitializedExecutionContext
            }
            ink_engine::OffChainError::UnregisteredChainExtension => {
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
                let acc: Vec<u8> = acc.into();
                AccountError::NoAccountForId(
                    scale::Decode::decode(&mut &acc[..]).expect("decoding failed"),
                )
            }
        }
    }
}

impl From<ink_engine::AccountError> for Error {
    fn from(account_error: ink_engine::AccountError) -> Self {
        Error::OffChain(OffChainError::Account(account_error.into()))
    }
}
