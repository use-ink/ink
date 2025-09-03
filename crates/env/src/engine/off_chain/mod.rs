// Copyright (C) Use Ink (UK) Ltd.
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
mod impls;
pub mod test_api;
mod types;

#[cfg(test)]
mod tests;

use super::OnInstance;
use crate::Error;

use derive_more::From;
use ink_engine::ext::Engine;
use ink_primitives::{
    AccountId,
    Address,
};

/// The off-chain environment.
pub struct EnvInstance {
    engine: Engine,
}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        use core::cell::RefCell;
        thread_local!(
            static INSTANCE: RefCell<EnvInstance> = RefCell::new(
                EnvInstance {
                    engine: Engine::new()
                }
            )
        );
        /*
         * This unsafe block is needed to be able to return a mut reference
         * while another mut reference is still borrowed, because now that
         * contracts can invoke other contracts some API functions are called
         * nested. This should be safe, as the object is in a TLS, so there's no
         * possibility of undefined behavior arising from race conditions.
         */
        INSTANCE.with(|instance| f(unsafe { &mut *instance.as_ptr() }))
    }
}

#[derive(Debug, From, PartialEq, Eq)]
pub enum OffChainError {
    Account(AccountError),
    #[from(ignore)]
    UninitializedBlocks,
    #[from(ignore)]
    UninitializedExecutionContext,
}

// todo rename
/// Errors encountered upon interacting with the accounts database.
#[derive(Debug, From, PartialEq, Eq)]
pub enum AccountError {
    Decoding(scale::Error),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(AccountId),
    NoContractForId(Address),
}
