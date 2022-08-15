use super::*;
use crate::{
    e2e::Signer,
    Environment,
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
use pallet_contracts_primitives::{
    ContractResult,
    ExecReturnValue,
    InstantiateReturnValue,
};
use sp_core::{
    Bytes,
    H256,
};
use subxt::{
    rpc::NumberOrHex,
    tx::{
        ExtrinsicParams,
        TxEvents,
    },
    OnlineClient,
};

/// The gas limit for contract instantiate and call dry runs.
const DRY_RUN_GAS_LIMIT: u64 = 500_000_000_000;

// TODO(https://github.com/paritytech/subxt/issues/640) Should be fetched automatically.
#[subxt::subxt(runtime_metadata_path = "metadata/contracts-node.scale")]
pub(super) mod api {}

/// A raw call to `pallet-contracts`'s `instantiate_with_code`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct InstantiateWithCode<B> {
    #[codec(compact)]
    value: B,
    #[codec(compact)]
    gas_limit: crate::types::Gas,
    storage_deposit_limit: Option<B>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
}

/// A raw call to `pallet-contracts`'s `call`.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct Call<C: subxt::Config, B> {
    dest: ::subxt::ext::sp_runtime::MultiAddress<C::AccountId, ()>,
    #[codec(compact)]
    value: B,
    #[codec(compact)]
    gas_limit: crate::types::Gas,
    storage_deposit_limit: Option<B>,
    data: Vec<u8>,
}

/// Result of a contract call dry run.
pub(super) type ContractDryCallResult<E> = ContractResult<
    Result<ExecReturnValue, serde_json::Value>,
    <E as Environment>::Balance,
>;

/// Result of a contract instantiation dry run.
pub(super) type ContractDryInstantiateResult<C, E> = ContractResult<
    Result<InstantiateReturnValue<<C as subxt::Config>::AccountId>, serde_json::Value>,
    <E as Environment>::Balance,
>;

/// A struct that encodes RPC parameters required to instantiate a new smart contract.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InstantiateRequest<C: subxt::Config> {
    origin: C::AccountId,
    value: NumberOrHex,
    gas_limit: NumberOrHex,
    storage_deposit_limit: Option<NumberOrHex>,
    code: Code,
    data: Bytes,
    salt: Bytes,
}

/// Reference to an existing code hash or a new Wasm module.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
enum Code {
    /// A Wasm module as raw bytes.
    Upload(Bytes),
    #[allow(unused)]
    /// The code hash of an on-chain Wasm blob.
    Existing(H256),
}

/// A struct that encodes RPC parameters required for a call to a smart contract.
///
/// Copied from [`pallet-contracts-rpc`].
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RpcCallRequest<C: subxt::Config> {
    origin: C::AccountId,
    dest: C::AccountId,
    value: NumberOrHex,
    gas_limit: NumberOrHex,
    storage_deposit_limit: Option<NumberOrHex>,
    input_data: Bytes,
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
    E::Balance:
        scale::Encode + TryFrom<NumberOrHex> + TryFrom<sp_rpc::number::NumberOrHex>,
    NumberOrHex: From<<E as Environment>::Balance>,

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
    ) -> ContractDryInstantiateResult<C, E> {
        let code = Code::Upload(code.into());
        let value = value.try_into().ok().unwrap();
        let call_request = InstantiateRequest::<C> {
            origin: signer.account_id().clone(),
            value,
            gas_limit: NumberOrHex::Number(DRY_RUN_GAS_LIMIT),
            storage_deposit_limit: storage_deposit_limit.map(|l| {
                l.try_into()
                    .ok()
                    .expect("unable to convert `storage_deposit_limit`")
            }),
            code,
            data: data.into(),
            salt: salt.into(),
        };

        let params = rpc_params![call_request];
        self.ws_client
            .request("contracts_instantiate", params)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `contracts_instantiate`: {:?}", err);
            })
    }

    /// Submits an extrinsic to instantiate a contract with the given code.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn instantiate_with_code(
        &self,
        value: E::Balance,
        gas_limit: crate::types::Gas,
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

        self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&call, signer)
            .await
            .map(|tx_progress| {
                log_info(&format!("signed and submitted tx with hash {:?}", tx_progress.extrinsic_hash()));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `sign_and_submit_then_watch_default`: {:?}",
                    err
                );
            })
            // TODO(#xxx) It should be configurable to use `.wait_for_finalized_success` instead.
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `wait_for_in_block`: {:?}",
                    err
                );
            })
            .fetch_events()
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `fetch_events`: {:?}",
                    err
                );
            })
    }

    /// Dry runs a call of the contract at `contract` with the given parameters.
    pub async fn call_dry_run(
        &self,
        contract: C::AccountId,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
        data: Vec<u8>,
    ) -> ContractDryCallResult<E> {
        let call_request = RpcCallRequest::<C> {
            origin: contract.clone(),
            dest: contract,
            value: value.try_into().ok().expect("unable to convert `value`"),
            gas_limit: DRY_RUN_GAS_LIMIT.into(),
            storage_deposit_limit: storage_deposit_limit.map(|l| {
                l.try_into()
                    .expect("unable to convert `storage_deposit_limit`")
            }),
            input_data: Bytes(data),
        };
        let params = rpc_params![call_request];
        self.ws_client
            .request("contracts_call", params)
            .await
            .unwrap_or_else(|err| {
                panic!("error on ws request `contracts_call`: {:?}", err);
            })
    }

    /// Submits an extrinsic to call a contract with the given parameters.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn call(
        &self,
        contract: sp_runtime::MultiAddress<C::AccountId, ()>,
        value: E::Balance,
        gas_limit: crate::types::Gas,
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

        self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&call, signer)
            .await
            .map(|tx_progress| {
                log_info(&format!("signed and submitted call with extrinsic hash {:?}", tx_progress.extrinsic_hash()));
                tx_progress
            })
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `sign_and_submit_then_watch_default`: {:?}",
                    err
                );
            })
            // TODO(#xxx) It should be configurable to use `.wait_for_finalized_success` instead.
            .wait_for_in_block()
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `wait_for_in_block`: {:?}",
                    err
                );
            })
            .fetch_events()
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "error on call `fetch_events`: {:?}",
                    err
                );
            })
    }
}
