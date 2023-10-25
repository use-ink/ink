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

use super::{
    log_info,
    sr25519,
    ContractExecResult,
    ContractInstantiateResult,
    Keypair,
};
use ink_env::Environment;

use core::marker::PhantomData;
use pallet_contracts_primitives::CodeUploadResult;
use sp_core::H256;
use subxt::{
    backend::{
        legacy::LegacyRpcMethods,
        rpc::RpcClient,
    },
    blocks::ExtrinsicEvents,
    config::ExtrinsicParams,
    ext::scale_encode,
    tx::Signer,
    utils::MultiAddress,
    OnlineClient,
};

/// Copied from `sp_weight` to additionally implement `scale_encode::EncodeAsType`.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Default,
    scale::Encode,
    scale::Decode,
    scale::MaxEncodedLen,
    scale_encode::EncodeAsType,
    serde::Serialize,
    serde::Deserialize,
)]
#[encode_as_type(crate_path = "subxt::ext::scale_encode")]
pub struct Weight {
    #[codec(compact)]
    /// The weight of computational time used based on some reference hardware.
    ref_time: u64,
    #[codec(compact)]
    /// The weight of storage space used by proof of validity.
    proof_size: u64,
}

impl From<sp_weights::Weight> for Weight {
    fn from(weight: sp_weights::Weight) -> Self {
        Self {
            ref_time: weight.ref_time(),
            proof_size: weight.proof_size(),
        }
    }
}

impl From<Weight> for sp_weights::Weight {
    fn from(weight: Weight) -> Self {
        sp_weights::Weight::from_parts(weight.ref_time, weight.proof_size)
    }
}

/// A raw call to `pallet-contracts`'s `instantiate_with_code`.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct InstantiateWithCode<E: Environment> {
    #[codec(compact)]
    value: E::Balance,
    gas_limit: Weight,
    storage_deposit_limit: Option<E::Balance>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `call`.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct Call<E: Environment> {
    dest: MultiAddress<E::AccountId, ()>,
    #[codec(compact)]
    value: E::Balance,
    gas_limit: Weight,
    storage_deposit_limit: Option<E::Balance>,
    data: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `call`.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct Transfer<E: Environment, C: subxt::Config> {
    dest: subxt::utils::Static<C::Address>,
    #[codec(compact)]
    value: E::Balance,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    scale::Decode,
    scale::Encode,
    scale_encode::EncodeAsType,
)]
#[encode_as_type(crate_path = "subxt::ext::scale_encode")]
pub enum Determinism {
    /// The execution should be deterministic and hence no indeterministic instructions
    /// are allowed.
    ///
    /// Dispatchables always use this mode in order to make on-chain execution
    /// deterministic.
    Enforced,
    /// Allow calling or uploading an indeterministic code.
    ///
    /// This is only possible when calling into `pallet-contracts` directly via
    /// [`crate::Pallet::bare_call`].
    ///
    /// # Note
    ///
    /// **Never** use this mode for on-chain execution.
    Relaxed,
}

/// A raw call to `pallet-contracts`'s `upload`.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct UploadCode<E: Environment> {
    code: Vec<u8>,
    storage_deposit_limit: Option<E::Balance>,
    determinism: Determinism,
}

/// A struct that encodes RPC parameters required to instantiate a new smart contract.
#[derive(serde::Serialize, scale::Encode)]
#[serde(rename_all = "camelCase")]
struct RpcInstantiateRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    value: E::Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
    code: Code,
    data: Vec<u8>,
    salt: Vec<u8>,
}

/// A struct that encodes RPC parameters required to upload a new smart contract.
#[derive(serde::Serialize, scale::Encode)]
#[serde(rename_all = "camelCase")]
struct RpcCodeUploadRequest<C: subxt::Config, E: Environment>
where
    E::Balance: serde::Serialize,
{
    origin: C::AccountId,
    code: Vec<u8>,
    storage_deposit_limit: Option<E::Balance>,
    determinism: Determinism,
}

/// A struct that encodes RPC parameters required for a call to a smart contract.
///
/// Copied from [`pallet-contracts-rpc`].
#[derive(serde::Serialize, scale::Encode)]
#[serde(rename_all = "camelCase")]
struct RpcCallRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    dest: E::AccountId,
    value: E::Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
    input_data: Vec<u8>,
}

/// Reference to an existing code hash or a new Wasm module.
#[derive(serde::Serialize, scale::Encode)]
#[serde(rename_all = "camelCase")]
enum Code {
    /// A Wasm module as raw bytes.
    Upload(Vec<u8>),
    #[allow(unused)]
    /// The code hash of an on-chain Wasm blob.
    Existing(H256),
}

/// Provides functions for interacting with the `pallet-contracts` API.
pub struct ContractsApi<C: subxt::Config, E: Environment> {
    pub rpc: LegacyRpcMethods<C>,
    pub client: OnlineClient<C>,
    _phantom: PhantomData<fn() -> (C, E)>,
}

impl<C, E> ContractsApi<C, E>
where
    C: subxt::Config,
    C::AccountId: From<sr25519::PublicKey> + serde::de::DeserializeOwned + scale::Codec,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::OtherParams: Default,

    E: Environment,
    E::Balance: scale::HasCompact + serde::Serialize,
{
    /// Creates a new [`ContractsApi`] instance.
    pub async fn new(rpc: RpcClient) -> Result<Self, subxt::Error> {
        let client = OnlineClient::<C>::from_rpc_client(rpc.clone()).await?;
        let rpc = LegacyRpcMethods::<C>::new(rpc);
        Ok(Self {
            rpc,
            client,
            _phantom: Default::default(),
        })
    }

    /// Attempt to transfer the `value` from `origin` to `dest`.
    ///
    /// Returns `Ok` on success, and a [`subxt::Error`] if the extrinsic is
    /// invalid (e.g. out of date nonce)
    pub async fn try_transfer_balance(
        &self,
        origin: &Keypair,
        dest: C::AccountId,
        value: E::Balance,
    ) -> Result<(), subxt::Error> {
        let call = subxt::tx::Payload::new(
            "Balances",
            "transfer_allow_death",
            Transfer::<E, C> {
                dest: subxt::utils::Static(dest.into()),
                value,
            },
        )
        .unvalidated();

        let account_id = <Keypair as Signer<C>>::account_id(origin);
        let account_nonce =
            self.get_account_nonce(&account_id)
                .await
                .unwrap_or_else(|err| {
                    panic!("error calling `get_account_nonce`: {err:?}");
                });

        self.client
            .tx()
            .create_signed_with_nonce(&call, origin, account_nonce, Default::default())
            .unwrap_or_else(|err| {
                panic!("error on call `create_signed_with_nonce`: {err:?}");
            })
            .submit_and_watch()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `submit_and_watch`: {err:?}");
            })
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `wait_for_in_block`: {err:?}");
            });

        Ok(())
    }

    /// Dry runs the instantiation of the given `code`.
    pub async fn instantiate_with_code_dry_run(
        &self,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
        code: Vec<u8>,
        data: Vec<u8>,
        salt: Vec<u8>,
        signer: &Keypair,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()> {
        let code = Code::Upload(code);
        let call_request = RpcInstantiateRequest::<C, E> {
            origin: Signer::<C>::account_id(signer),
            value,
            gas_limit: None,
            storage_deposit_limit,
            code,
            data,
            salt,
        };
        let func = "ContractsApi_instantiate";
        let params = scale::Encode::encode(&call_request);
        let bytes = self
            .rpc
            .state_call(func, Some(&params), None)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `contracts_instantiate`: {err:?}");
            });
        scale::Decode::decode(&mut bytes.as_ref()).unwrap_or_else(|err| {
            panic!("decoding ContractInstantiateResult failed: {err}")
        })
    }

    /// Sign and submit an extrinsic with the given call payload.
    pub async fn submit_extrinsic<Call>(
        &self,
        call: &Call,
        signer: &Keypair,
    ) -> ExtrinsicEvents<C>
    where
        Call: subxt::tx::TxPayload,
    {
        let account_id = <Keypair as Signer<C>>::account_id(signer);
        let account_nonce =
            self.get_account_nonce(&account_id)
                .await
                .unwrap_or_else(|err| {
                    panic!("error calling `get_account_nonce`: {err:?}");
                });

        self.client
            .tx()
            .create_signed_with_nonce(call, signer, account_nonce, Default::default())
            .unwrap_or_else(|err| {
                panic!("error on call `create_signed_with_nonce`: {err:?}");
            })
            .submit_and_watch()
            .await
            .map(|tx_progress| {
                log_info(&format!(
                    "signed and submitted tx with hash {:?}",
                    tx_progress.extrinsic_hash()
                ));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!("error on call `submit_and_watch`: {err:?}");
            })
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `wait_for_in_block`: {err:?}");
            })
            .fetch_events()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `fetch_events`: {err:?}");
            })
    }

    /// Return the hash of the *best* block
    pub async fn best_block(&self) -> C::Hash {
        self.rpc
            .chain_get_block_hash(None)
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `chain_get_block_hash`: {err:?}");
            })
            .unwrap_or_else(|| {
                panic!("error on call `chain_get_block_hash`: no best block found");
            })
    }

    /// Return the account nonce at the *best* block for an account ID.
    async fn get_account_nonce(
        &self,
        account_id: &C::AccountId,
    ) -> Result<u64, subxt::Error> {
        let best_block = self.best_block().await;
        let account_nonce = self
            .client
            .blocks()
            .at(best_block)
            .await?
            .account_nonce(account_id)
            .await?;
        Ok(account_nonce)
    }

    /// Submits an extrinsic to instantiate a contract with the given code.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    #[allow(clippy::too_many_arguments)]
    pub async fn instantiate_with_code(
        &self,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
        code: Vec<u8>,
        data: Vec<u8>,
        salt: Vec<u8>,
        signer: &Keypair,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::Payload::new(
            "Contracts",
            "instantiate_with_code",
            InstantiateWithCode::<E> {
                value,
                gas_limit,
                storage_deposit_limit,
                code,
                data,
                salt,
            },
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Dry runs the upload of the given `code`.
    pub async fn upload_dry_run(
        &self,
        signer: &Keypair,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CodeUploadResult<E::Hash, E::Balance> {
        let call_request = RpcCodeUploadRequest::<C, E> {
            origin: Signer::<C>::account_id(signer),
            code,
            storage_deposit_limit,
            determinism: Determinism::Enforced,
        };
        let func = "ContractsApi_upload_code";
        let params = scale::Encode::encode(&call_request);
        let bytes = self
            .rpc
            .state_call(func, Some(&params), None)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `upload_code`: {err:?}");
            });
        scale::Decode::decode(&mut bytes.as_ref())
            .unwrap_or_else(|err| panic!("decoding CodeUploadResult failed: {err}"))
    }

    /// Submits an extrinsic to upload a given code.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn upload(
        &self,
        signer: &Keypair,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::Payload::new(
            "Contracts",
            "upload_code",
            UploadCode::<E> {
                code,
                storage_deposit_limit,
                determinism: Determinism::Enforced,
            },
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Dry runs a call of the contract at `contract` with the given parameters.
    pub async fn call_dry_run(
        &self,
        origin: C::AccountId,
        dest: E::AccountId,
        input_data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractExecResult<E::Balance, ()> {
        let call_request = RpcCallRequest::<C, E> {
            origin,
            dest,
            value,
            gas_limit: None,
            storage_deposit_limit,
            input_data,
        };
        let func = "ContractsApi_call";
        let params = scale::Encode::encode(&call_request);
        let bytes = self
            .rpc
            .state_call(func, Some(&params), None)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `contracts_call`: {err:?}");
            });
        scale::Decode::decode(&mut bytes.as_ref())
            .unwrap_or_else(|err| panic!("decoding ContractExecResult failed: {err}"))
    }

    /// Submits an extrinsic to call a contract with the given parameters.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn call(
        &self,
        contract: MultiAddress<E::AccountId, ()>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
        data: Vec<u8>,
        signer: &Keypair,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::Payload::new(
            "Contracts",
            "call",
            Call::<E> {
                dest: contract,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            },
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Submit an extrinsic `call_name` for the `pallet_name`.
    /// The `call_data` is a `Vec<subxt::dynamic::Value>` that holds
    /// a representation of some value.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn runtime_call<'a>(
        &self,
        signer: &Keypair,
        pallet_name: &'a str,
        call_name: &'a str,
        call_data: Vec<subxt::dynamic::Value>,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::dynamic::tx(pallet_name, call_name, call_data);

        self.submit_extrinsic(&call, signer).await
    }
}
