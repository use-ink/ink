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
mod error;
pub mod events;
mod node_proc;
#[cfg(feature = "sandbox")]
mod sandbox_client;
mod subxt_client;
mod xts;

pub use crate::contract_build::build_root_and_contract_dependencies;
pub use backend::{
    BuilderClient,
    ChainBackend,
    ContractsBackend,
    E2EBackend,
};
pub use backend_calls::{
    CallBuilder,
    InstantiateBuilder,
};
pub use client_utils::ContractsRegistry;
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
pub use pallet_revive::evm::CallTrace;
#[cfg(feature = "sandbox")]
pub use sandbox_client::{
    preset,
    Client as SandboxClient,
};
pub use sp_keyring::Sr25519Keyring;
pub use subxt::{
    self,
    backend::rpc::RpcClient,
};
pub use subxt_client::{
    CallBuilderFinal,
    Client,
    Error,
};
pub use subxt_signer::{
    self,
    sr25519::{
        self,
        dev::*,
        Keypair,
    },
};
pub use tokio;
pub use tracing_subscriber;

#[cfg(feature = "sandbox")]
pub use ink_sandbox::DefaultSandbox;

use ink::codegen::ContractCallBuilder;
use ink_env::{
    call::FromAddr,
    ContractEnv,
    Environment,
};
use ink_primitives::{
    Address,
    DepositLimit,
    H256,
};
pub use sp_weights::Weight;
use std::{
    cell::RefCell,
    sync::Once,
};
use xts::ReviveApi;

use ink_primitives::types::AccountIdMapper;
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
pub fn account_id(account: Sr25519Keyring) -> ink_primitives::AccountId {
    ink_primitives::AccountId::try_from(account.to_account_id().as_ref())
        .expect("account keyring has a valid account id")
}

/// Returns the [`ink::Address`] for a given keyring account.
///
/// # Developer Note
///
/// We take the `AccountId` and return only the first twenty bytes, this
/// is what `pallet-revive` does as well.
pub fn address<E: Environment>(account: Sr25519Keyring) -> Address {
    AccountIdMapper::to_address(account.to_account_id().as_ref())
}

/// Creates a call builder for `Contract`, based on an account id.
pub fn create_call_builder<Contract>(
    acc_id: Address,
) -> <Contract as ContractCallBuilder>::Type
where
    <Contract as ContractEnv>::Env: Environment,
    Contract: ContractCallBuilder,
    Contract: ContractEnv,
    Contract::Type: FromAddr,
{
    <<Contract as ContractCallBuilder>::Type as FromAddr>::from_addr(acc_id)
}

fn balance_to_deposit_limit<E: Environment>(
    b: <E as Environment>::Balance,
) -> DepositLimit<<E as Environment>::Balance> {
    DepositLimit::Balance(b)
}

fn deposit_limit_to_balance<E: Environment>(
    l: DepositLimit<<E as Environment>::Balance>,
) -> <E as Environment>::Balance {
    match l {
        DepositLimit::Balance(l) => l,
        // todo
        DepositLimit::Unchecked => panic!("`Unchecked` is not supported"),
    }
}
