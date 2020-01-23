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
        AccountsDb,
        AccountError,
        CodeDb,
        ExecContext,
    },
    typed_encoded::{TypedEncoded, TypedEncodedError, Result},
    types::{
        OffAccountId,
        OffBalance,
        OffHash,
        OffMoment,
        OffBlockNumber,
        OffCall,
    },
};
use super::OnInstance;
use core::cell::RefCell;

/// The off-chain environment.
///
/// Mainly used for off-chain testing.
pub struct EnvInstance {
    /// The accounts database of the environment.
    accounts: AccountsDb,
    /// Uploaded Wasm contract codes.
    codes: CodeDb,
    /// Current execution context and context.
    session: Option<ExecContext>,
}

impl EnvInstance {
    /// Creates a new uninitialized off-chain environment.
    pub fn uninitialized() -> Self {
        Self {
            accounts: AccountsDb::new(),
            codes: CodeDb::new(),
            session: None,
        }
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
