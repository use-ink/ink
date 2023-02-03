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

use super::{
    builders::Message,
    log_info,
    sr25519,
    ContractExecResult,
    ContractInstantiateResult,
    Signer,
};
use ink_env::Environment;

use core::marker::PhantomData;
use pallet_contracts_primitives::CodeUploadResult;
use sp_core::{
    Bytes,
    H256,
};
use sp_weights::Weight;
use subxt::{
    blocks::ExtrinsicEvents,
    config::ExtrinsicParams,
    rpc_params,
    tx,
    OnlineClient,
};

/// A raw call to `pallet-contracts`'s `instantiate_with_code`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct InstantiateWithCode<B> {
    #[codec(compact)]
    value: B,
    gas_limit: Weight,
    storage_deposit_limit: Option<B>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `call`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct Call<E: Environment, B> {
    dest: sp_runtime::MultiAddress<E::AccountId, ()>,
    #[codec(compact)]
    value: B,
    gas_limit: Weight,
    storage_deposit_limit: Option<B>,
    data: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `call`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct Transfer<E: Environment, C: subxt::Config> {
    dest: C::Address,
    #[codec(compact)]
    value: E::Balance,
}

#[derive(
    Debug, Clone, Copy, scale::Encode, scale::Decode, PartialEq, Eq, serde::Serialize,
)]
pub enum Determinism {
    /// The execution should be deterministic and hence no indeterministic instructions are
    /// allowed.
    ///
    /// Dispatchables always use this mode in order to make on-chain execution deterministic.
    Deterministic,
    /// Allow calling or uploading an indeterministic code.
    ///
    /// This is only possible when calling into `pallet-contracts` directly via
    /// [`crate::Pallet::bare_call`].
    ///
    /// # Note
    ///
    /// **Never** use this mode for on-chain execution.
    AllowIndeterminism,
}

/// A raw call to `pallet-contracts`'s `upload`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct UploadCode<B> {
    code: Vec<u8>,
    storage_deposit_limit: Option<B>,
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
    pub client: OnlineClient<C>,
    _phantom: PhantomData<fn() -> (C, E)>,
}

impl<C, E> ContractsApi<C, E>
where
    C: subxt::Config,
    C::AccountId: serde::de::DeserializeOwned,
    C::AccountId: scale::Codec,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C::Index, C::Hash>>::OtherParams: Default,

    E: Environment,
    E::Balance: scale::HasCompact + serde::Serialize,
{
    /// Creates a new [`ContractsApi`] instance.
    pub async fn new(client: OnlineClient<C>) -> Self {
        Self {
            client,
            _phantom: Default::default(),
        }
    }

    /// Attempt to transfer the `value` from `origin` to `dest`.
    ///
    /// Returns `Ok` on success, and a [`subxt::Error`] if the extrinsic is
    /// invalid (e.g. out of date nonce)
    pub async fn try_transfer_balance(
        &self,
        origin: &Signer<C>,
        dest: C::AccountId,
        value: E::Balance,
    ) -> Result<(), subxt::Error> {
        let call = subxt::tx::StaticTxPayload::new(
            "Balances",
            "transfer",
            Transfer::<E, C> {
                dest: dest.into(),
                value,
            },
            Default::default(),
        )
        .unvalidated();

        let tx_progress = self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&call, origin)
            .await?;

        tx_progress.wait_for_in_block().await.unwrap_or_else(|err| {
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
        signer: &Signer<C>,
    ) -> ContractInstantiateResult<C::AccountId, E::Balance> {
        let code = Code::Upload(code);
        let call_request = RpcInstantiateRequest::<C, E> {
            origin: subxt::tx::Signer::account_id(signer).clone(),
            value,
            gas_limit: None,
            storage_deposit_limit,
            code,
            data,
            salt,
        };
        let func = "ContractsApi_instantiate";
        let params = rpc_params![func, Bytes(scale::Encode::encode(&call_request))];
        let bytes: Bytes = self
            .client
            .rpc()
            .request("state_call", params)
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
        signer: &Signer<C>,
    ) -> ExtrinsicEvents<C>
    where
        Call: tx::TxPayload,
    {
        self.client
            .tx()
            .sign_and_submit_then_watch_default(call, signer)
            .await
            .map(|tx_progress| {
                log_info(&format!(
                    "signed and submitted tx with hash {:?}",
                    tx_progress.extrinsic_hash()
                ));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!("error on call `sign_and_submit_then_watch_default`: {err:?}");
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
        signer: &Signer<C>,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::StaticTxPayload::new(
            "Contracts",
            "instantiate_with_code",
            InstantiateWithCode::<E::Balance> {
                value,
                gas_limit,
                storage_deposit_limit,
                code,
                data,
                salt,
            },
            Default::default(),
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Dry runs the upload of the given `code`.
    pub async fn upload_dry_run(
        &self,
        signer: &Signer<C>,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CodeUploadResult<E::Hash, E::Balance> {
        let call_request = RpcCodeUploadRequest::<C, E> {
            origin: subxt::tx::Signer::account_id(signer).clone(),
            code,
            storage_deposit_limit,
            determinism: Determinism::Deterministic,
        };
        let func = "ContractsApi_upload_code";
        let params = rpc_params![func, Bytes(scale::Encode::encode(&call_request))];
        let bytes: Bytes = self
            .client
            .rpc()
            .request("state_call", params)
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
        signer: &Signer<C>,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::StaticTxPayload::new(
            "Contracts",
            "upload_code",
            UploadCode::<E::Balance> {
                code,
                storage_deposit_limit,
                determinism: Determinism::Deterministic,
            },
            Default::default(),
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Dry runs a call of the contract at `contract` with the given parameters.
    pub async fn call_dry_run<RetType>(
        &self,
        origin: C::AccountId,
        message: &Message<E, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractExecResult<E::Balance> {
        let call_request = RpcCallRequest::<C, E> {
            origin,
            dest: message.account_id().clone(),
            value,
            gas_limit: None,
            storage_deposit_limit,
            input_data: message.exec_input().to_vec(),
        };
        let func = "ContractsApi_call";
        let params = rpc_params![func, Bytes(scale::Encode::encode(&call_request))];
        let bytes: Bytes = self
            .client
            .rpc()
            .request("state_call", params)
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
        contract: sp_runtime::MultiAddress<E::AccountId, ()>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
        data: Vec<u8>,
        signer: &Signer<C>,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::StaticTxPayload::new(
            "Contracts",
            "call",
            Call::<E, E::Balance> {
                dest: contract,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            },
            Default::default(),
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }
}
