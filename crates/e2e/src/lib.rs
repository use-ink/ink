// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Module for the logic behind ink!'s End-to-End testing framework.

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]

mod backend;
mod backend_calls;
mod builders;
mod client_utils;
mod contract_build;
mod contract_results;
#[cfg(feature = "drink")]
mod drink_client;
mod error;
pub mod events;
mod node_proc;
mod subxt_client;
mod xts;

pub use crate::contract_build::{
    build_root_and_additional_contracts,
    build_root_and_contract_dependencies,
};
pub use backend::{
    ChainBackend,
    ContractsBackend,
    E2EBackend,
};
pub use backend_calls::{
    CallBuilder,
    InstantiateBuilder,
};
pub use contract_results::{
    CallDryRunResult,
    CallResult,
    InstantiateDryRunResult,
    InstantiationResult,
    UploadResult,
};
pub use ink_e2e_macro::test;
pub use node_proc::{
    TestNodeProcess,
    TestNodeProcessBuilder,
};
pub use sp_core::H256;
pub use sp_keyring::AccountKeyring;
pub use subxt::{
    self,
    backend::rpc::RpcClient,
};
pub use subxt_client::{
    CallBuilderFinal,
    Client,
    Error,
};
pub use subxt_signer::sr25519::{
    self,
    dev::*,
    Keypair,
};
pub use tokio;
pub use tracing_subscriber;
#[cfg(feature = "drink")]
pub use {
    drink::runtime::MinimalRuntime,
    drink_client::Client as DrinkClient,
};

use pallet_contracts_primitives::{
    ContractExecResult,
    ContractInstantiateResult,
};
use std::{
    cell::RefCell,
    sync::Once,
};
use xts::ContractsApi;

pub use subxt::PolkadotConfig;

/// We use this to only initialize `env_logger` once.
pub static INIT: Once = Once::new();

// We save the name of the currently executing test here as a mean
// of prefixing log entries to make it easier pinning them to tests.
thread_local! {
    /// This prefix will be used for log output. It is set by each
    /// `#[ink_e2e::test]` with the function name as String.
    /// This way it is possible to distinguish the lines in stdout
    /// and stderr, to still know which line belongs to which test.
    pub static LOG_PREFIX: RefCell<String> = RefCell::new(String::from("no prefix set"));
}

/// Returns the name of the test which is currently executed.
pub fn log_prefix() -> String {
    LOG_PREFIX.with(|log_prefix| log_prefix.borrow().clone())
}

/// Writes `msg` to stdout.
pub fn log_info(msg: &str) {
    tracing::info!("[{}] {}", log_prefix(), msg);
}

/// Writes `msg` to stderr.
pub fn log_error(msg: &str) {
    tracing::error!("[{}] {}", log_prefix(), msg);
}

/// Get an ink! [`ink_primitives::AccountId`] for a given keyring account.
pub fn account_id(account: AccountKeyring) -> ink_primitives::AccountId {
    ink_primitives::AccountId::try_from(account.to_account_id().as_ref())
        .expect("account keyring has a valid account id")
}

/// Builds a contract and imports its scaffolded structure as a module.
#[macro_export]
macro_rules! build {
        ($($arg:tt)*) => (
            ink_e2e::smart_bench_macro::contract!($($arg)*)
        );
}
