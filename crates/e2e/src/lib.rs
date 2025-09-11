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
    Client as SandboxClient,
    preset,
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
        Keypair,
        dev::*,
    },
};
pub use tokio;
pub use tracing;
pub use tracing_subscriber;

#[cfg(feature = "sandbox")]
pub use ink_sandbox::DefaultSandbox;

use ink::codegen::ContractCallBuilder;
use ink_env::{
    ContractEnv,
    Environment,
    call::FromAddr,
};
use ink_primitives::{
    Address,
    DepositLimit,
    H256,
    types::AccountIdMapper,
};
pub use sp_weights::Weight;
use std::{
    cell::RefCell,
    sync::Once,
};
use xts::ReviveApi;

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

/// Returns the [`ink::Address`] for a given account id.
///
/// # Developer Note
///
/// We take the `AccountId` and return only the first twenty bytes, this
/// is what `pallet-revive` does as well.
pub fn address_from_account_id<AccountId: AsRef<[u8]>>(account_id: AccountId) -> Address {
    AccountIdMapper::to_address(account_id.as_ref())
}

/// Returns the [`ink::Address`] for a given `Keypair`.
///
/// # Developer Note
///
/// We take the `AccountId` and return only the first twenty bytes, this
/// is what `pallet-revive` does as well.
pub fn address_from_keypair<AccountId: From<[u8; 32]> + AsRef<[u8]>>(
    keypair: &Keypair,
) -> Address {
    let account_id: AccountId = keypair_to_account(keypair);
    address_from_account_id(account_id)
}

/// Transforms a `Keypair` into an account id.
pub fn keypair_to_account<AccountId: From<[u8; 32]>>(keypair: &Keypair) -> AccountId {
    AccountId::from(keypair.public_key().0)
}

/// Creates a call builder for `Contract`, based on an account id.
pub fn create_call_builder<Contract>(
    acc_id: Address,
) -> <Contract as ContractCallBuilder>::Type<ink::env::DefaultAbi>
where
    <Contract as ContractEnv>::Env: Environment,
    Contract: ContractCallBuilder + ContractEnv,
    <Contract as ContractCallBuilder>::Type<ink::env::DefaultAbi>: FromAddr,
{
    <<Contract as ContractCallBuilder>::Type<ink::env::DefaultAbi> as FromAddr>::from_addr(
        acc_id,
    )
}

/// Creates a call builder for `Contract` for the specified ABI, based on an account id.
pub fn create_call_builder_abi<Contract, Abi>(
    acc_id: Address,
) -> <Contract as ContractCallBuilder>::Type<Abi>
where
    <Contract as ContractEnv>::Env: Environment,
    Contract: ContractCallBuilder + ContractEnv,
    <Contract as ContractCallBuilder>::Type<Abi>: FromAddr,
{
    <<Contract as ContractCallBuilder>::Type<Abi> as FromAddr>::from_addr(acc_id)
}

/// Transforms `Option<<E as Environment>::Balance>>` into `DepositLimit`.
///
/// This function must only be used for dry-runs, a `None` will
/// become an unrestricted deposit limit (`DepositLimit::UnsafeOnlyForDryRun`).
fn balance_to_deposit_limit_dry_run<E: Environment>(
    b: Option<<E as Environment>::Balance>,
) -> DepositLimit<<E as Environment>::Balance> {
    match b {
        Some(v) => DepositLimit::Balance(v),
        None => DepositLimit::UnsafeOnlyForDryRun,
    }
}

/// Transforms `Option<<E as Environment>::Balance>>` into `DepositLimit`.
/// This function must be used for submitting extrinsics on-chain.
///
/// Panics if `limit` is `None`. Make sure to execute a dry-run
/// beforehand and use the `storage_deposit_limit` result of it here.
fn balance_to_deposit_limit<E: Environment>(
    limit: Option<<E as Environment>::Balance>,
) -> DepositLimit<<E as Environment>::Balance> {
    match limit {
        Some(val) => DepositLimit::Balance(val),
        None => panic!("Deposit limit must be specified for on-chain submissions."),
    }
}

/// Transforms `DepositLimit<<E as Environment>::Balance>` into `<E as
/// Environment>::Balance>`.
///
/// Panics if `limit` is unrestricted (`DepositLimit::UnsafeOnlyForDryRun`).
fn deposit_limit_to_balance<E: Environment>(
    limit: DepositLimit<<E as Environment>::Balance>,
) -> <E as Environment>::Balance {
    match limit {
        DepositLimit::Balance(val) => val,
        DepositLimit::UnsafeOnlyForDryRun => {
            panic!("Unrestricted deposit limit not allowed for balance conversion!")
        }
    }
}
