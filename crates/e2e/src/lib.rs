// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

mod builders;
mod client;
mod default_accounts;
mod node_proc;
mod xts;

pub use builders::{
    build_message,
    MessageBuilder,
};
pub use client::{
    CallDryRunResult,
    CallResult,
    Client,
    Error,
    InstantiationResult,
    UploadResult,
};
pub use default_accounts::*;
pub use env_logger;
pub use ink_e2e_macro::test;
pub use node_proc::{
    TestNodeProcess,
    TestNodeProcessBuilder,
};
pub use sp_core::H256;
pub use sp_keyring::AccountKeyring;
pub use subxt::{
    self,
    tx::PairSigner,
};
pub use tokio;

use pallet_contracts_primitives::{
    CodeUploadResult,
    ContractExecResult,
    ContractInstantiateResult,
};
use sp_core::sr25519;
use std::{
    cell::RefCell,
    sync::Once,
};
use xts::ContractsApi;

/// Default set of commonly used types by Substrate runtimes.
#[cfg(feature = "std")]
pub enum SubstrateConfig {}

#[cfg(feature = "std")]
impl subxt::Config for SubstrateConfig {
    type Index = u32;
    type Hash = sp_core::H256;
    type Hasher = subxt::config::substrate::BlakeTwo256;
    type AccountId = subxt::config::substrate::AccountId32;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type Header = subxt::config::substrate::SubstrateHeader<
        u32,
        subxt::config::substrate::BlakeTwo256,
    >;
    type Signature = sp_runtime::MultiSignature;
    type ExtrinsicParams = subxt::config::substrate::SubstrateExtrinsicParams<Self>;
}

/// Default set of commonly used types by Polkadot nodes.
#[cfg(feature = "std")]
pub type PolkadotConfig = subxt::config::WithExtrinsicParams<
    SubstrateConfig,
    subxt::config::polkadot::PolkadotExtrinsicParams<SubstrateConfig>,
>;

/// Signer that is used throughout the E2E testing.
///
/// The E2E testing can only be used with nodes that support `sr25519`
/// cryptography.
pub type Signer<C> = PairSigner<C, sr25519::Pair>;

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
    log::info!("[{}] {}", log_prefix(), msg);
}

/// Writes `msg` to stderr.
pub fn log_error(msg: &str) {
    log::error!("[{}] {}", log_prefix(), msg);
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
