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
    log_info,
    sr25519,
    ContractExecResult,
    ContractInstantiateResult,
    IdentifyAccount,
    Signer,
    Verify,
};
use ink_env::{
    Environment,
    Gas,
};

use core::marker::PhantomData;
use jsonrpsee::{
    core::client::ClientT,
    rpc_params,
    ws_client::{
        WsClient,
        WsClientBuilder,
    },
};
use pallet_contracts_primitives::CodeUploadResult;
use sp_core::{
    Bytes,
    H256,
};
use subxt::{
    tx::{
        ExtrinsicParams,
        TxEvents,
    },
    OnlineClient,
};

/// The gas limit for contract instantiate and call dry runs.
const DRY_RUN_GAS_LIMIT: u64 = 500_000_000_000;

// TODO(#1422) Should be fetched automatically.
#[subxt::subxt(
    crate = "crate::subxt",
    runtime_metadata_path = "metadata/contracts-node.scale"
)]
pub(super) mod api {}

/// A raw call to `pallet-contracts`'s `instantiate_with_code`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct InstantiateWithCode<B> {
    #[codec(compact)]
    value: B,
    #[codec(compact)]
    gas_limit: Gas,
    storage_deposit_limit: Option<B>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `call`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct Call<C: subxt::Config, B> {
    dest: sp_runtime::MultiAddress<C::AccountId, ()>,
    #[codec(compact)]
    value: B,
    #[codec(compact)]
    gas_limit: Gas,
    storage_deposit_limit: Option<B>,
    data: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `upload`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct UploadCode<B> {
    code: Vec<u8>,
    storage_deposit_limit: Option<B>,
}

/// A struct that encodes RPC parameters required to instantiate a new smart contract.
#[derive(serde::Serialize, scale::Encode)]
#[serde(rename_all = "camelCase")]
struct RpcInstantiateRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    value: E::Balance,
    gas_limit: Gas,
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
}

/// A struct that encodes RPC parameters required for a call to a smart contract.
///
/// Copied from [`pallet-contracts-rpc`].
#[derive(serde::Serialize, scale::Encode)]
#[serde(rename_all = "camelCase")]
struct RpcCallRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    dest: C::AccountId,
    value: E::Balance,
    gas_limit: Gas,
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
    ws_client: WsClient,
    _phantom: PhantomData<fn() -> (C, E)>,
}

impl<C, E> ContractsApi<C, E>
where
    C: subxt::Config,
    C::AccountId: Into<C::Address> + serde::de::DeserializeOwned,
    <C::ExtrinsicParams as ExtrinsicParams<C::Index, C::Hash>>::OtherParams: Default,

    C::Signature: From<sr25519::Signature>,
    <C::Signature as Verify>::Signer: From<sr25519::Public>,
    <C::Signature as Verify>::Signer:
        From<sr25519::Public> + IdentifyAccount<AccountId = C::AccountId>,
    sr25519::Signature: Into<C::Signature>,

    E: Environment,
    E::Balance: scale::Encode + serde::Serialize,

    Call<C, E::Balance>: scale::Encode,
    InstantiateWithCode<E::Balance>: scale::Encode,
{
    /// Creates a new [`ContractsApi`] instance.
    pub async fn new(client: OnlineClient<C>, url: &str) -> Self {
        let ws_client =
            WsClientBuilder::default()
                .build(&url)
                .await
                .unwrap_or_else(|err| {
                    panic!("error on ws request: {:?}", err);
                });

        Self {
            client,
            ws_client,
            _phantom: Default::default(),
        }
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
            origin: signer.account_id().clone(),
            value,
            gas_limit: DRY_RUN_GAS_LIMIT,
            storage_deposit_limit,
            code,
            data,
            salt,
        };
        let func = "ContractsApi_instantiate";
        let params = rpc_params![func, Bytes(scale::Encode::encode(&call_request))];
        let bytes: Bytes = self
            .ws_client
            .request("state_call", params)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `contracts_instantiate`: {:?}", err);
            });
        scale::Decode::decode(&mut bytes.as_ref())
            .unwrap_or_else(|err| panic!("decoding failed: {}", err))
    }

    /// Submits an extrinsic to instantiate a contract with the given code.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    #[allow(clippy::too_many_arguments)]
    pub async fn instantiate_with_code(
        &self,
        value: E::Balance,
        gas_limit: Gas,
        storage_deposit_limit: Option<E::Balance>,
        code: Vec<u8>,
        data: Vec<u8>,
        salt: Vec<u8>,
        signer: &Signer<C>,
    ) -> TxEvents<C> {
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

        self.client
            .tx()
            .sign_and_submit_then_watch_default(&call, signer)
            .await
            .map(|tx_progress| {
                log_info(&format!(
                    "signed and submitted tx with hash {:?}",
                    tx_progress.extrinsic_hash()
                ));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `sign_and_submit_then_watch_default`: {:?}",
                    err
                );
            })
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `wait_for_in_block`: {:?}", err);
            })
            .fetch_events()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `fetch_events`: {:?}", err);
            })
    }

    /// Dry runs the upload of the given `code`.
    pub async fn upload_dry_run(
        &self,
        signer: &Signer<C>,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CodeUploadResult<C::Hash, E::Balance> {
        let call_request = RpcCodeUploadRequest::<C, E> {
            origin: signer.account_id().clone(),
            code,
            storage_deposit_limit,
        };
        let func = "ContractsApi_upload_code";
        let params = rpc_params![func, Bytes(scale::Encode::encode(&call_request))];
        let bytes: Bytes = self
            .ws_client
            .request("state_call", params)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `upload_code`: {:?}", err);
            });
        scale::Decode::decode(&mut bytes.as_ref())
            .unwrap_or_else(|err| panic!("decoding failed: {}", err))
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
    ) -> TxEvents<C> {
        let call = subxt::tx::StaticTxPayload::new(
            "Contracts",
            "upload_code",
            UploadCode::<E::Balance> {
                code,
                storage_deposit_limit,
            },
            Default::default(),
        )
        .unvalidated();

        self.client
            .tx()
            .sign_and_submit_then_watch_default(&call, signer)
            .await
            .map(|tx_progress| {
                log_info(&format!(
                    "signed and submitted tx with hash {:?}",
                    tx_progress.extrinsic_hash()
                ));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `sign_and_submit_then_watch_default`: {:?}",
                    err
                );
            })
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `wait_for_in_block`: {:?}", err);
            })
            .fetch_events()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `fetch_events`: {:?}", err);
            })
    }

    /// Dry runs a call of the contract at `contract` with the given parameters.
    pub async fn call_dry_run(
        &self,
        contract: C::AccountId,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
        input_data: Vec<u8>,
    ) -> ContractExecResult<E::Balance> {
        let call_request = RpcCallRequest::<C, E> {
            origin: contract.clone(),
            dest: contract,
            value,
            gas_limit: DRY_RUN_GAS_LIMIT,
            storage_deposit_limit,
            input_data,
        };
        let func = "ContractsApi_call";
        let params = rpc_params![func, Bytes(scale::Encode::encode(&call_request))];
        let bytes: Bytes = self
            .ws_client
            .request("state_call", params)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `contracts_call`: {:?}", err);
            });
        scale::Decode::decode(&mut bytes.as_ref())
            .unwrap_or_else(|err| panic!("decoding failed: {}", err))
    }

    /// Submits an extrinsic to call a contract with the given parameters.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn call(
        &self,
        contract: sp_runtime::MultiAddress<C::AccountId, ()>,
        value: E::Balance,
        gas_limit: Gas,
        storage_deposit_limit: Option<E::Balance>,
        data: Vec<u8>,
        signer: &Signer<C>,
    ) -> TxEvents<C> {
        let call = subxt::tx::StaticTxPayload::new(
            "Contracts",
            "call",
            Call::<C, E::Balance> {
                dest: contract,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            },
            Default::default(),
        )
        .unvalidated();

        self.client
            .tx()
            .sign_and_submit_then_watch_default(&call, signer)
            .await
            .map(|tx_progress| {
                log_info(&format!(
                    "signed and submitted call with extrinsic hash {:?}",
                    tx_progress.extrinsic_hash()
                ));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `sign_and_submit_then_watch_default`: {:?}",
                    err
                );
            })
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `wait_for_in_block`: {:?}", err);
            })
            .fetch_events()
            .await
            .unwrap_or_else(|err| {
                panic!("error on call `fetch_events`: {:?}", err);
            })
    }
}
