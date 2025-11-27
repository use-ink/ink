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
};

/// Native sr25519 keypair type re-exported for convenience.
pub type Sr25519Keypair = subxt_signer::sr25519::Keypair;

/// Native Ethereum keypair type re-exported for convenience.
pub type EthKeypair = eth::EthKeypair;

/// Ethereum keypair types for use with pallet-revive.
///
/// Using Ethereum keypairs is the recommended approach for interacting with
/// pallet-revive contracts. Unlike Substrate keypairs, Ethereum keypairs:
/// - Don't require explicit account mapping
/// - Have perfect address roundtrip (H160 → AccountId32 → H160)
/// - Work seamlessly with MetaMask and other Ethereum wallets
///
/// # Example
/// ```ignore
/// use ink_e2e::eth::{self, dev::alith};
///
/// let keypair = alith();
/// let address = keypair.address(); // Native H160 Ethereum address
/// ```
pub mod eth {
    pub use subxt_signer::eth::{
        Keypair as EthKeypair,
        PublicKey as EthPublicKey,
        Signature as EthSignature,
        dev,
    };

    // Re-export common dev accounts at module level for convenience
    pub use subxt_signer::eth::dev::{
        alith,
        baltathar,
        charleth,
        dorothy,
        ethan,
        faith,
    };
}

/// Re-export sr25519 signer types and dev accounts for callers that still need
/// direct access to the raw Substrate keys.
pub mod sr25519 {
    pub use subxt_signer::sr25519::{
        Keypair,
        PublicKey,
        Signature,
        dev,
    };
}
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
pub use sp_weights::Weight;
use std::{
    cell::RefCell,
    sync::Once,
};
use crate::sr25519::dev as sr25519_dev;
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

/// Unified keypair type that can represent either a Substrate sr25519 keypair
/// or an Ethereum ECDSA keypair. This lets e2e tests choose which signing scheme
/// to use while keeping the high-level API stable.
#[derive(Clone)]
pub enum Keypair {
    Sr25519(Sr25519Keypair),
    Eth(EthKeypair),
}

impl Keypair {
    /// Returns the AccountId32 bytes for this keypair.
    /// - sr25519: raw public key bytes
    /// - eth: fallback format `[H160][0xEE;12]`
    pub fn account_id_bytes(&self) -> [u8; 32] {
        match self {
            Keypair::Sr25519(kp) => kp.public_key().0,
            Keypair::Eth(kp) => {
                let eth_address = kp.public_key().to_account_id();
                let mut account_bytes = [0xEE_u8; 32];
                account_bytes[..20].copy_from_slice(&eth_address.0);
                account_bytes
            }
        }
    }

    pub fn is_eth(&self) -> bool {
        matches!(self, Keypair::Eth(_))
    }

    pub fn as_sr25519(&self) -> Option<&Sr25519Keypair> {
        match self {
            Keypair::Sr25519(kp) => Some(kp),
            _ => None,
        }
    }

    pub fn as_eth(&self) -> Option<&EthKeypair> {
        match self {
            Keypair::Eth(kp) => Some(kp),
            _ => None,
        }
    }
}

impl From<Sr25519Keypair> for Keypair {
    fn from(value: Sr25519Keypair) -> Self {
        Keypair::Sr25519(value)
    }
}

impl From<EthKeypair> for Keypair {
    fn from(value: EthKeypair) -> Self {
        Keypair::Eth(value)
    }
}

/// Sr25519 dev accounts (Substrate keyring), wrapped into the unified `Keypair`.
pub fn alice() -> Keypair {
    Keypair::from(sr25519_dev::alice())
}
pub fn bob() -> Keypair {
    Keypair::from(sr25519_dev::bob())
}
pub fn charlie() -> Keypair {
    Keypair::from(sr25519_dev::charlie())
}
pub fn dave() -> Keypair {
    Keypair::from(sr25519_dev::dave())
}
pub fn eve() -> Keypair {
    Keypair::from(sr25519_dev::eve())
}
pub fn ferdie() -> Keypair {
    Keypair::from(sr25519_dev::ferdie())
}
pub fn one() -> Keypair {
    Keypair::from(sr25519_dev::one())
}
pub fn two() -> Keypair {
    Keypair::from(sr25519_dev::two())
}

/// Ethereum dev accounts wrapped into the unified `Keypair`.
pub fn alith() -> Keypair {
    Keypair::from(eth::dev::alith())
}
pub fn baltathar() -> Keypair {
    Keypair::from(eth::dev::baltathar())
}
pub fn charleth() -> Keypair {
    Keypair::from(eth::dev::charleth())
}
pub fn dorothy() -> Keypair {
    Keypair::from(eth::dev::dorothy())
}
pub fn ethan() -> Keypair {
    Keypair::from(eth::dev::ethan())
}
pub fn faith() -> Keypair {
    Keypair::from(eth::dev::faith())
}

/// Backwards-compatible dev module to mirror the old `subxt_signer::sr25519::dev` API
/// while returning the unified `Keypair` wrapper. Includes both sr25519 and Ethereum
/// dev accounts.
pub mod dev {
    pub use crate::{
        alice,
        alith,
        baltathar,
        bob,
        charleth,
        charlie,
        dave,
        dorothy,
        ethan,
        eve,
        faith,
        ferdie,
        one,
        two,
    };
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
    AccountId::from(keypair.account_id_bytes())
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

/// Extension trait for converting various types to Address (H160).
pub trait IntoAddress {
    /// Convert to an Address (H160).
    fn address(&self) -> Address;
}

impl IntoAddress for Keypair {
    fn address(&self) -> Address {
        AccountIdMapper::to_address(&self.account_id_bytes())
    }
}

impl IntoAddress for ink_primitives::AccountId {
    fn address(&self) -> Address {
        let bytes = *AsRef::<[u8; 32]>::as_ref(self);
        AccountIdMapper::to_address(&bytes)
    }
}

impl IntoAddress for eth::EthKeypair {
    /// Returns the native Ethereum H160 address for this keypair.
    ///
    /// This is derived using the standard Ethereum method:
    /// `keccak256(uncompressed_pubkey[1..65])[12..32]`
    ///
    /// Unlike Substrate keypairs, this address has a perfect roundtrip:
    /// - H160 → AccountId32 (fallback with 0xEE suffix) → H160 (strips suffix)
    fn address(&self) -> Address {
        // eth::PublicKey::to_account_id() returns AccountId20 which is the H160
        // derived via keccak256(pubkey[1..65])[12..32]
        let account_id_20 = self.public_key().to_account_id();
        Address::from(account_id_20.0)
    }
}

/// Trait for keypairs that can be used to sign transactions in e2e tests.
///
/// This trait abstracts over both Sr25519 (Substrate) and ECDSA (Ethereum) keypairs,
/// allowing the e2e testing framework to work seamlessly with either.
///
/// # Implementors
///
/// - [`Keypair`] (Sr25519): Traditional Substrate keypairs from `subxt_signer::sr25519`
/// - [`eth::EthKeypair`] (ECDSA): Ethereum keypairs from `subxt_signer::eth`
///
/// # Example
///
/// ```ignore
/// use ink_e2e::{Signer, alice, eth::alith};
///
/// // Both Sr25519 and Ethereum keypairs can be used
/// let sr25519_account: [u8; 32] = alice().account_id();
/// let eth_account: [u8; 32] = alith().account_id();
/// ```
pub trait Signer: Send + Sync {
    /// Returns the 32-byte account ID for this keypair.
    ///
    /// For Sr25519 keypairs, this is the raw public key.
    /// For Ethereum keypairs, this is the fallback format: `[H160][0xEE; 12]`.
    fn account_id(&self) -> [u8; 32];
}

impl<C> subxt::tx::Signer<C> for Keypair
where
    C: subxt::Config,
    C::AccountId: From<[u8; 32]>,
    C::Signature: From<sr25519::Signature> + From<eth::EthSignature>,
{
    fn account_id(&self) -> C::AccountId {
        C::AccountId::from(self.account_id_bytes())
    }

    fn sign(&self, payload: &[u8]) -> C::Signature {
        match self {
            Keypair::Sr25519(kp) => kp.sign(payload).into(),
            Keypair::Eth(kp) => kp.sign(payload).into(),
        }
    }
}

impl Signer for Keypair {
    fn account_id(&self) -> [u8; 32] {
        self.account_id_bytes()
    }
}

impl Signer for Sr25519Keypair {
    fn account_id(&self) -> [u8; 32] {
        self.public_key().0
    }
}

impl Signer for eth::EthKeypair {
    /// Returns the fallback AccountId32 format for Ethereum keypairs.
    ///
    /// Format: `[H160 (20 bytes)][0xEE repeated 12 times]`
    ///
    /// This format is automatically recognized as "Ethereum-derived" by pallet-revive,
    /// which means no explicit account mapping is required.
    fn account_id(&self) -> [u8; 32] {
        let eth_address = self.public_key().to_account_id();
        let mut account_bytes = [0xEE_u8; 32];
        account_bytes[..20].copy_from_slice(&eth_address.0);
        account_bytes
    }
}
