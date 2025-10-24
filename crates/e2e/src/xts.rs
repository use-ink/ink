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
    Keypair,
    log_info,
    sr25519,
};
use ink_env::Environment;

use crate::contract_results::{
    ContractExecResultFor,
    ContractInstantiateResultFor,
};
use core::marker::PhantomData;
use funty::Fundamental;
use ink_primitives::Address;
use ink_revive_types::{
    CodeUploadResult,
    evm::{
        CallTrace,
        CallTracerConfig,
        Trace,
        TracerType,
    },
};
use sp_core::H256;
use sp_runtime::OpaqueExtrinsic;
use subxt::{
    OnlineClient,
    backend::{
        legacy::LegacyRpcMethods,
        rpc::RpcClient,
    },
    blocks::ExtrinsicEvents,
    config::{
        DefaultExtrinsicParams,
        DefaultExtrinsicParamsBuilder,
        ExtrinsicParams,
        HashFor,
        Header,
    },
    ext::{
        scale_encode,
        subxt_core::tx::Transaction,
    },
    tx::{
        Signer,
        SubmittableTransaction,
        TxStatus,
    },
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
///
/// See <https://github.com/use-ink/polkadot-sdk/blob/c40b36c3a7c208f9a6837b80812473af3d9ba7f7/substrate/frame/revive/src/lib.rs>.
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
///
/// See <https://github.com/use-ink/polkadot-sdk/blob/c40b36c3a7c208f9a6837b80812473af3d9ba7f7/substrate/frame/revive/src/lib.rs>.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct Call<E: Environment> {
    dest: Address,
    #[codec(compact)]
    value: E::Balance,
    gas_limit: Weight,
    #[codec(compact)]
    storage_deposit_limit: E::Balance,
    data: Vec<u8>,
}

/// A raw call to `pallet-revive`'s `map_account`.
///
/// See <https://github.com/use-ink/polkadot-sdk/blob/c40b36c3a7c208f9a6837b80812473af3d9ba7f7/substrate/frame/revive/src/lib.rs>.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct MapAccount {}

/// A raw call to `pallet-balances`'s `transfer`.
#[derive(Debug, scale::Decode, scale::Encode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct Transfer<E: Environment, C: subxt::Config> {
    dest: subxt::utils::Static<C::Address>,
    #[codec(compact)]
    value: E::Balance,
}

/// A raw call to `pallet-revive`'s `remove_code`.
///
/// See <https://github.com/use-ink/polkadot-sdk/blob/c40b36c3a7c208f9a6837b80812473af3d9ba7f7/substrate/frame/revive/src/lib.rs>.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct RemoveCode {
    code_hash: H256,
}

/// A raw call to `pallet-revive`'s `upload_code`.
///
/// See <https://github.com/use-ink/polkadot-sdk/blob/c40b36c3a7c208f9a6837b80812473af3d9ba7f7/substrate/frame/revive/src/lib.rs>.
#[derive(Debug, scale::Encode, scale::Decode, scale_encode::EncodeAsType)]
#[encode_as_type(trait_bounds = "", crate_path = "subxt::ext::scale_encode")]
pub struct UploadCode<E: Environment> {
    code: Vec<u8>,
    #[codec(compact)]
    storage_deposit_limit: E::Balance,
}

/// A struct that encodes RPC parameters required to instantiate a new smart contract.
#[derive(scale::Encode)]
struct RpcInstantiateRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    value: E::Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
    code: Code,
    data: Vec<u8>,
    salt: Option<[u8; 32]>,
}

/// A struct that encodes RPC parameters required to upload a new smart contract.
#[derive(scale::Encode)]
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
struct RpcCallRequest<C: subxt::Config, E: Environment> {
    origin: C::AccountId,
    dest: Address,
    value: E::Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
    input_data: Vec<u8>,
}

/// Reference to an existing code hash or a new contract binary.
#[derive(scale::Encode)]
enum Code {
    /// A contract binary as raw bytes.
    Upload(Vec<u8>),
    #[allow(unused)]
    /// The code hash of an on-chain contract blob.
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
    E::Balance: scale::HasCompact + serde::Serialize + std::fmt::Debug,
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
    pub async fn transfer_allow_death(
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
        storage_deposit_limit: Option<E::Balance>,
        code: Vec<u8>,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        signer: &Keypair,
    ) -> ContractInstantiateResultFor<E> {
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
            panic!("decoding `ContractInstantiateResult` failed: {err}")
        })
    }

    /// todo
    pub async fn create_extrinsic<Call>(
        &self,
        call: &Call,
        signer: &Keypair,
    ) -> SubmittableTransaction<C, OnlineClient<C>>
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
        self.client
            .tx()
            .create_partial_offline(call, params.into())
            .unwrap_or_else(|err| {
                panic!("error on call `create_signed_with_nonce`: {err:?}");
            })
            .sign(signer)
    }

    /// Sign and submit an extrinsic with the given call payload.
    pub async fn submit_extrinsic<Call>(
        &self,
        call: &Call,
        signer: &Keypair,
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>)
    where
        Call: subxt::tx::Payload,
    {
        // we have to retrieve the current block hash here. we use it later in this
        // function when retrieving the log. the extrinsic is dry-run for tracing
        // the log. if we were to use the latest block the extrinsic would already
        // have been executed and we would get an error.
        let parent_hash = self.best_block().await;

        let mut tx = self
            .create_extrinsic(call, signer)
            .await
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
        // We require this because we use `ink-node` as our development
        // node, which does not currently support finality, so we just want to
        // wait until it is included in a block.
        while let Some(status) = tx.next().await {
            match status.unwrap_or_else(|err| {
                panic!("error subscribing to tx status: {err:?}");
            }) {
                TxStatus::InBestBlock(tx_in_block)
                | TxStatus::InFinalizedBlock(tx_in_block) => {
                    let events = tx_in_block.fetch_events().await.unwrap_or_else(|err| {
                        panic!("error on call `fetch_events`: {err:?}");
                    });
                    let trace = self
                        .trace(
                            tx_in_block.block_hash(),
                            Some(tx_in_block.extrinsic_hash()),
                            parent_hash,
                            None,
                        )
                        .await;
                    return (events, trace)
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

    /// todo
    pub async fn trace(
        &self,
        block_hash: HashFor<C>,
        extrinsic_hash: Option<HashFor<C>>,
        parent_hash: HashFor<C>,
        extrinsic: Option<Vec<u8>>,
    ) -> Option<CallTrace> {
        // todo move below to its own function
        let block_details = self
            .rpc
            .chain_get_block(Some(block_hash))
            .await
            .expect("no block found")
            .expect("no block details found");
        let header = block_details.block.header;
        let mut exts: Vec<OpaqueExtrinsic> = block_details
            .block
            .extrinsics
            .clone()
            .into_iter()
            .filter_map(|e| scale::Decode::decode(&mut &e[..]).ok())
            .collect::<Vec<_>>();

        // todo
        let tx_index: usize = match (extrinsic_hash, extrinsic) {
            (Some(hash), None) => {
                block_details
                    .block
                    .extrinsics
                    .iter()
                    .cloned()
                    .enumerate()
                    .find_map(|(index, ext)| {
                        let hash_ext = Transaction::<C>::from_bytes(ext.0)
                            .hash_with(self.client.hasher());
                        if hash_ext == hash {
                            return Some(index);
                        }
                        None
                    })
                    .expect("the extrinsic hash was not found in the block")
            }
            (None, Some(extrinsic)) => {
                exts.push(
                    OpaqueExtrinsic::from_bytes(&extrinsic[..])
                        .expect("OpaqueExtrinsic cannot be created"),
                );
                exts.len() - 1
            }
            _ => panic!("pattern error"),
        };

        let tracer_type = TracerType::CallTracer(Some(CallTracerConfig::default()));
        let func = "ReviveApi_trace_tx";

        let params =
            scale::Encode::encode(&((header, exts), tx_index.as_u32(), tracer_type));

        let bytes = self
            .rpc
            .state_call(func, Some(&params), Some(parent_hash))
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "error on ws request `trace_tx`: {err:?}\n\n{:#}",
                    format!("{err}").trim_start_matches("RPC error: ")
                );
            });

        let trace: Option<Trace> = scale::Decode::decode(&mut bytes.as_ref())
            .unwrap_or_else(|err| panic!("decoding `trace_tx` result failed: {err}"));

        match trace {
            Some(Trace::Call(trace)) => Some(trace),
            _ => None,
        }
    }

    /// Return the hash of the *best* block
    pub async fn best_block(&self) -> HashFor<C> {
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
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>) {
        let call = subxt::tx::DefaultPayload::new(
            "Revive",
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
    ) -> CodeUploadResult<E::Balance> {
        let call_request = RpcCodeUploadRequest::<C, E> {
            origin: Signer::<C>::account_id(signer),
            code,
            storage_deposit_limit,
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
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>) {
        let call = subxt::tx::DefaultPayload::new(
            "Revive",
            "upload_code",
            UploadCode::<E> {
                code,
                storage_deposit_limit,
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
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>) {
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
        dest: Address,
        data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
        signer: &Keypair,
    ) -> (ContractExecResultFor<E>, Option<CallTrace>) {
        let origin = Signer::<C>::account_id(signer);
        let call_request = RpcCallRequest::<C, E> {
            origin,
            dest,
            value,
            gas_limit: Some(Weight {
                ref_time: u64::MAX,
                proof_size: u64::MAX,
            }),
            storage_deposit_limit: storage_deposit_limit.clone(),
            input_data: data.clone(),
        };
        let func = "ReviveApi_call";
        let params = scale::Encode::encode(&call_request);
        let bytes = self
            .rpc
            .state_call(func, Some(&params), None)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `ReviveApi_call`: {err:?}");
            });
        let dry_run_result: ContractExecResultFor<E> =
            scale::Decode::decode(&mut bytes.as_ref()).unwrap_or_else(|err| {
                panic!("decoding `ContractExecResult` failed: {err}")
            });

        // Even if the `storage_deposit_limit` to this function was set as `Unchecked`,
        // we still take the return value of the dry run for submitting the extrinsic
        // that will take effect.
        let storage_deposit_limit = match storage_deposit_limit {
            None => dry_run_result.storage_deposit.charge_or_zero(),
            Some(limit) => limit,
        };

        let call = subxt::tx::DefaultPayload::new(
            "Revive",
            "call",
            crate::xts::Call::<E> {
                dest,
                value,
                gas_limit: Weight {
                    ref_time: dry_run_result.gas_required.ref_time(),
                    proof_size: dry_run_result.gas_required.proof_size(),
                },
                storage_deposit_limit,
                data,
            },
        )
        .unvalidated();
        let xt = self.create_extrinsic(&call, signer).await;
        let block_hash = self.best_block().await;
        let block_details = self
            .rpc
            .chain_get_block(Some(block_hash))
            .await
            .expect("no block found")
            .expect("no block details found");
        let block_number: u64 = block_details.block.header.number().into();
        let parent_hash = self
            .rpc
            .chain_get_block_hash(Some((block_number - 1u64).into()))
            .await
            .expect("no block hash found")
            .expect("no block details found");

        let trace = self
            .trace(block_hash, None, parent_hash, Some(xt.into_encoded()))
            .await;

        (dry_run_result, trace)
    }

    /// Submits an extrinsic to call a contract with the given parameters.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    /// todo the API for `call_dry_run` should mirror that of `call`
    pub async fn call(
        &self,
        contract: Address,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
        data: Vec<u8>,
        signer: &Keypair,
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>) {
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

    /// Maps the `signer` to an `H160` account.
    ///
    /// This is a `pallet-revive` concept, whereby a storage entry is created on-chain.
    /// The entry maps the account id from `signer` to an `H160` account. This is
    /// a necessity for any operation interacting with the contracts part of
    /// `pallet-revive`.
    pub async fn map_account(
        &self,
        signer: &Keypair,
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>) {
        // todo check if the account is unmapped! otherwise
        // we submit a costly extrinisc which is guaranteed to fail.
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
    ) -> (ExtrinsicEvents<C>, Option<CallTrace>) {
        let call = subxt::dynamic::tx(pallet_name, call_name, call_data);
        self.submit_extrinsic(&call, signer).await
    }
}
