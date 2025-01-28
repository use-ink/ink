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

use super::{
    log_info,
    sr25519,
    Keypair,
};
use ink_env::Environment;

use crate::contract_results::{
    ContractExecResultFor,
    ContractInstantiateResultForBar,
};
use core::marker::PhantomData;
use ink_primitives::DepositLimit;
use pallet_revive::{
    evm::H160,
    CodeUploadResult,
};
use sp_core::H256;
use subxt::{
    backend::{
        legacy::LegacyRpcMethods,
        rpc::RpcClient,
    },
    blocks::ExtrinsicEvents,
    config::{
        DefaultExtrinsicParams,
        DefaultExtrinsicParamsBuilder,
        ExtrinsicParams,
    },
    ext::scale_encode,
    tx::{
        Signer,
        TxStatus,
    },
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

/// A raw call to `pallet-revive`'s `instantiate_with_code`.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct InstantiateWithCode<E: Environment> {
    #[codec(compact)]
    value: E::Balance,
    gas_limit: Weight,
    #[codec(compact)]
    storage_deposit_limit: E::Balance,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Option<[u8; 32]>,
}

/// A raw call to `pallet-revive`'s `call`.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct Call<E: Environment> {
    dest: H160,
    #[codec(compact)]
    value: E::Balance,
    gas_limit: Weight,
    #[codec(compact)]
    storage_deposit_limit: E::Balance,
    data: Vec<u8>,
}

/// A raw call to `pallet-revive`'s `map_account`.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct MapAccount {}

/// A raw call to `pallet-revive`'s `call`.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct Transfer<E: Environment, C: subxt::Config> {
    dest: subxt::utils::Static<C::Address>,
    #[codec(compact)]
    value: E::Balance,
}

/// A raw call to `pallet-revive`'s `remove_code`.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct RemoveCode {
    code_hash: H256,
}

/// A raw call to `pallet-revive`'s `upload_code`.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct UploadCode<E: Environment> {
    code: Vec<u8>,
    #[codec(compact)]
    storage_deposit_limit: E::Balance,
}

/// A struct that encodes RPC parameters required to instantiate a new smart contract.
#[derive(scale::Encode)]
// todo: #[derive(serde::Serialize, scale::Encode)]
// todo: #[serde(rename_all = "camelCase")]
struct RpcInstantiateRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    value: E::Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: DepositLimit<E::Balance>,
    code: Code,
    data: Vec<u8>,
    salt: Option<[u8; 32]>,
}

/// A struct that encodes RPC parameters required to upload a new smart contract.
#[derive(scale::Encode)]
// todo: #[derive(serde::Serialize, scale::Encode)]
// todo: #[serde(rename_all = "camelCase")]
struct RpcCodeUploadRequest<C: subxt::Config, E: Environment>
where
    E::Balance: serde::Serialize,
{
    origin: C::AccountId,
    code: Vec<u8>,
    storage_deposit_limit: Option<E::Balance>,
}

/// A struct that encodes RPC parameters required for a call to a smart contract.
#[derive(scale::Encode)]
// todo: #[derive(serde::Serialize, scale::Encode)]
// todo: #[serde(rename_all = "camelCase")]
struct RpcCallRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    dest: H160,
    value: E::Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: DepositLimit<E::Balance>,
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

/// Provides functions for interacting with the `pallet-revive` API.
pub struct ReviveApi<C: subxt::Config, E: Environment> {
    pub rpc: LegacyRpcMethods<C>,
    pub client: OnlineClient<C>,
    _phantom: PhantomData<fn() -> (C, E)>,
}

impl<C, E> ReviveApi<C, E>
where
    C: subxt::Config,
    C::AccountId: From<sr25519::PublicKey> + serde::de::DeserializeOwned + scale::Codec,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::Params:
        From<<DefaultExtrinsicParams<C> as ExtrinsicParams<C>>::Params>,

    E: Environment,
    E::Balance: scale::HasCompact + serde::Serialize,
{
    /// Creates a new [`ReviveApi`] instance.
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
        let call = subxt::tx::DefaultPayload::new(
            "Balances",
            "transfer_allow_death",
            Transfer::<E, C> {
                dest: subxt::utils::Static(dest.into()),
                value,
            },
        )
        .unvalidated();

        let _ = self.submit_extrinsic(&call, origin).await;

        Ok(())
    }

    /// Dry runs the instantiation of the given `code`.
    pub async fn instantiate_with_code_dry_run(
        &self,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
        code: Vec<u8>,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        signer: &Keypair,
    ) -> ContractInstantiateResultForBar<E> {
        // todo map_account beforehand?
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
        let func = "ReviveApi_instantiate";
        let params = scale::Encode::encode(&call_request);
        let bytes = self
            .rpc
            .state_call(func, Some(&params), None)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `revive_instantiate`: {err:?}");
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
        Call: subxt::tx::Payload,
    {
        let account_id = <Keypair as Signer<C>>::account_id(signer);
        let account_nonce =
            self.get_account_nonce(&account_id)
                .await
                .unwrap_or_else(|err| {
                    panic!("error calling `get_account_nonce`: {err:?}");
                });

        let params = DefaultExtrinsicParamsBuilder::new()
            .nonce(account_nonce)
            .build();
        let mut tx = self
            .client
            .tx()
            .create_signed_offline(call, signer, params.into())
            .unwrap_or_else(|err| {
                panic!("error on call `create_signed_with_nonce`: {err:?}");
            })
            .submit_and_watch()
            .await
            .inspect(|tx_progress| {
                log_info(&format!(
                    "signed and submitted tx with hash {:?}",
                    tx_progress.extrinsic_hash()
                ));
            })
            .unwrap_or_else(|err| {
                panic!("error on call `submit_and_watch`: {err:?}");
            });

        // Below we use the low level API to replicate the `wait_for_in_block` behaviour
        // which was removed in subxt 0.33.0. See https://github.com/paritytech/subxt/pull/1237.
        //
        // We require this because we use `substrate-contracts-node` as our development
        // node, which does not currently support finality, so we just want to
        // wait until it is included in a block.
        while let Some(status) = tx.next().await {
            match status.unwrap_or_else(|err| {
                panic!("error subscribing to tx status: {err:?}");
            }) {
                TxStatus::InBestBlock(tx_in_block)
                | TxStatus::InFinalizedBlock(tx_in_block) => {
                    return tx_in_block.fetch_events().await.unwrap_or_else(|err| {
                        panic!("error on call `fetch_events`: {err:?}");
                    })
                }
                TxStatus::Error { message } => {
                    panic!("TxStatus::Error: {message:?}");
                }
                TxStatus::Invalid { message } => {
                    panic!("TxStatus::Invalid: {message:?}");
                }
                TxStatus::Dropped { message } => {
                    panic!("TxStatus::Dropped: {message:?}");
                }
                _ => continue,
            }
        }
        panic!("Error waiting for tx status")
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
        storage_deposit_limit: E::Balance,
        code: Vec<u8>,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        signer: &Keypair,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::DefaultPayload::new(
            "Revive",
            "instantiate_with_code",
            InstantiateWithCode::<E> {
                value,
                gas_limit,
                storage_deposit_limit, // todo
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
        // todo
        _storage_deposit_limit: E::Balance,
    ) -> CodeUploadResult<E::Balance> {
        let call_request = RpcCodeUploadRequest::<C, E> {
            origin: Signer::<C>::account_id(signer),
            code,
            //storage_deposit_limit,
            storage_deposit_limit: None,
        };
        let func = "ReviveApi_upload_code";
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
        storage_deposit_limit: E::Balance,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::DefaultPayload::new(
            "Revive",
            "upload_code",
            UploadCode::<E> {
                code,
                storage_deposit_limit,
                //storage_deposit_limit: None
            },
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Submits an extrinsic to remove the code at the given hash.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn remove_code(
        &self,
        signer: &Keypair,
        code_hash: H256,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::DefaultPayload::new(
            "Revive",
            "remove_code",
            RemoveCode { code_hash },
        )
        .unvalidated();

        self.submit_extrinsic(&call, signer).await
    }

    /// Dry runs a call of the contract at `contract` with the given parameters.
    pub async fn call_dry_run(
        &self,
        origin: C::AccountId,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        _storage_deposit_limit: E::Balance, // todo
    ) -> ContractExecResultFor<E> {
        let call_request = RpcCallRequest::<C, E> {
            origin,
            dest,
            value,
            gas_limit: None,
            storage_deposit_limit: DepositLimit::Unchecked,
            input_data,
        };
        let func = "ReviveApi_call";
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
        contract: H160,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
        data: Vec<u8>,
        signer: &Keypair,
    ) -> ExtrinsicEvents<C> {
        let call = subxt::tx::DefaultPayload::new(
            "Revive",
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

    /// todo
    /// Submits an extrinsic to call a contract with the given parameters.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn map_account(&self, signer: &Keypair) -> ExtrinsicEvents<C> {
        let call = subxt::tx::DefaultPayload::new("Revive", "map_account", MapAccount {})
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
