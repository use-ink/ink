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
mod macros;
mod node_proc;
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
pub use builders::{
    CreateBuilderPartial,
    constructor_exec_input,
};
pub use client_utils::{
    ContractsRegistry,
    code_hash,
    salt,
};
pub use contract_results::{
    BareInstantiationResult,
    CallDryRunResult,
    CallResult,
    ContractExecResultFor,
    ContractResult,
    InstantiateDryRunResult,
    InstantiationResult,
    UploadResult,
};
pub use ink_e2e_macro::test;
pub use ink_revive_types::evm::CallTrace;
pub use macros::{
    ContractEventReader,
    assert_last_event_internal,
};
pub use node_proc::{
    TestNodeProcess,
    TestNodeProcessBuilder,
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

use ink::codegen::ContractCallBuilder;
use ink_env::{
    ContractEnv,
    Environment,
    call::FromAddr,
};
use ink_primitives::{
    Address,
    H256,
    types::AccountIdMapper,
};
use sp_core::crypto::AccountId32;
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

/// Trait for converting various types into an `AccountId`.
///
/// This enables generic functions to accept multiple account representations
/// (e.g., `Keypair`, `AccountId32`, `ink_primitives::AccountId`) without
/// requiring callers to perform manual conversions.
///
/// Implementations extract the underlying 32-byte public key and convert it
/// to the target `AccountId` type.
pub trait IntoAccountId<TargetAccountId> {
    /// Converts this type into the target account ID.
    fn into_account_id(self) -> TargetAccountId;
}

impl IntoAccountId<AccountId32> for AccountId32 {
    fn into_account_id(self) -> AccountId32 {
        self
    }
}

impl IntoAccountId<AccountId32> for &AccountId32 {
    fn into_account_id(self) -> AccountId32 {
        self.clone()
    }
}

impl<AccountId> IntoAccountId<AccountId> for &ink_primitives::AccountId
where
    AccountId: From<[u8; 32]>,
{
    fn into_account_id(self) -> AccountId {
        AccountId::from(*AsRef::<[u8; 32]>::as_ref(self))
    }
}

impl<AccountId> IntoAccountId<AccountId> for &Keypair
where
    AccountId: From<[u8; 32]>,
{
    fn into_account_id(self) -> AccountId {
        AccountId::from(self.public_key().0)
    }
}

/// Trait for converting various types to an EVM-compatible `Address` (H160).
///
/// The conversion uses [`AccountIdMapper::to_address`] which applies different
/// strategies based on the account type:
/// - Ethereum-derived accounts (last 12 bytes are `0xEE`): extracts the first 20 bytes
/// - Sr25519-derived accounts: computes keccak256 hash and takes the last 20 bytes
pub trait IntoAddress {
    /// Converts this type to an EVM-compatible address.
    fn address(&self) -> Address;
}

impl IntoAddress for Keypair {
    fn address(&self) -> Address {
        AccountIdMapper::to_address(&self.public_key().0)
    }
}

impl IntoAddress for ink_primitives::AccountId {
    fn address(&self) -> Address {
        let bytes = *AsRef::<[u8; 32]>::as_ref(self);
        AccountIdMapper::to_address(&bytes)
    }
}

impl IntoAddress for AccountId32 {
    fn address(&self) -> Address {
        AccountIdMapper::to_address(self.as_ref())
    }
}
