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

#[cfg(feature = "std")]
use std::fmt::Debug;
use std::path::PathBuf;

use super::{
    H256,
    InstantiateDryRunResult,
    Keypair,
    ReviveApi,
    builders::{
        CreateBuilderPartial,
        constructor_exec_input,
    },
    deposit_limit_to_balance,
    events::{
        CodeStoredEvent,
        EventWithTopics,
    },
    log_error,
    log_info,
    sr25519,
};
use crate::{
    ContractsBackend,
    E2EBackend,
    backend::{
        BuilderClient,
        ChainBackend,
    },
    client_utils::{
        ContractsRegistry,
        salt,
    },
    contract_results::{
        BareInstantiationResult,
        CallDryRunResult,
        CallResult,
        ContractResult,
        UploadResult,
    },
    error::DryRunError,
    events,
    events::ContractInstantiatedEvent,
};
use ink::H160;
use ink_env::{
    Environment,
    call::{
        Call,
        ExecutionInput,
        utils::{
            DecodeMessageResult,
            ReturnType,
            Set,
        },
    },
};
use ink_primitives::{
    DepositLimit,
    abi::AbiEncodeWith,
};
use jsonrpsee::core::async_trait;
use pallet_revive::evm::CallTrace;
use scale::{
    Decode,
    Encode,
};
use sp_weights::Weight;
use subxt::{
    blocks::ExtrinsicEvents,
    config::{
        DefaultExtrinsicParams,
        ExtrinsicParams,
        HashFor,
    },
    error::DispatchError,
    events::EventDetails,
    ext::scale_value::{
        Composite,
        Value,
        ValueDef,
    },
    storage::dynamic,
    tx::Signer,
};

pub type Error = crate::error::Error<DispatchError>;

/// Represents an initialized contract message builder.
pub type CallBuilderFinal<E, Args, RetType, Abi> = ink_env::call::CallBuilder<
    E,
    Set<Call>,
    Set<ExecutionInput<Args, Abi>>,
    Set<ReturnType<RetType>>,
>;

/// The `Client` takes care of communicating with the node.
///
/// This node's RPC interface will be used for instantiating the contract
/// and interacting with it .
pub struct Client<C, E>
where
    C: subxt::Config,
    E: Environment,
{
    api: ReviveApi<C, E>,
    contracts: ContractsRegistry,
    url: String,
}

impl<C, E> Client<C, E>
where
    C: subxt::Config,
    C::AccountId:
        From<sr25519::PublicKey> + scale::Codec + serde::de::DeserializeOwned + Debug,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::Params:
        From<<DefaultExtrinsicParams<C> as ExtrinsicParams<C>>::Params>,
    E: Environment,
    E::AccountId: Debug,
    E::EventRecord: Debug,
    E::Balance: Debug + scale::HasCompact + serde::Serialize,
    H256: Debug + scale::Encode,
{
    /// Creates a new [`Client`] instance using a `subxt` client.
    pub async fn new<P: Into<PathBuf>>(
        client: subxt::backend::rpc::RpcClient,
        contracts: impl IntoIterator<Item = P>,
        url: String,
    ) -> Result<Self, subxt::Error> {
        Ok(Self {
            api: ReviveApi::new(client).await?,
            contracts: ContractsRegistry::new(contracts),
            url,
        })
    }

    /// Executes an `upload` call and captures the resulting events.
    async fn exec_upload(
        &mut self,
        signer: &Keypair,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, ExtrinsicEvents<C>>, Error> {
        let dry_run = self
            .api
            .upload_dry_run(signer, code.clone(), storage_deposit_limit)
            .await;
        log_info(&format!("upload dry run: {dry_run:?}"));
        if let Err(err) = dry_run {
            let dispatch_err = self.runtime_dispatch_error_to_subxt_dispatch_error(&err);
            return Err(Error::UploadDryRun(dispatch_err))
        }

        let storage_deposit_limit = match storage_deposit_limit {
            None => {
                dry_run
                    .as_ref()
                    .expect("would have returned already")
                    .deposit
            }
            Some(limit) => limit,
        };

        let (tx_events, trace) =
            self.api.upload(signer, code, storage_deposit_limit).await;

        let mut hash = None;
        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if let Some(uploaded) =
                evt.as_event::<CodeStoredEvent>().unwrap_or_else(|err| {
                    panic!("event conversion to `Uploaded` failed: {err:?}");
                })
            {
                log_info(&format!(
                    "contract was uploaded with hash {:?}",
                    uploaded.code_hash
                ));
                hash = Some(uploaded.code_hash);
                break
            } else if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;

                log_error(&format!(
                    "extrinsic for upload failed: {dispatch_error} {trace:?}"
                ));
                return Err(Error::UploadExtrinsic(dispatch_error, trace))
            }
        }

        // todo still up to date?
        // The `pallet-revive` behavior is that if the code was already stored on the
        // chain we won't get an event with the hash, but the extrinsic will still
        // succeed. We then don't error (`cargo-contract` would), but instead
        // return the hash from the dry-run.
        let code_hash = match hash {
            Some(hash) => hash,
            None => {
                dry_run
                    .as_ref()
                    .unwrap_or_else(|err| panic!("must have worked: {err:?}"))
                    .code_hash
            }
        };

        Ok(UploadResult {
            dry_run,
            code_hash,
            events: tx_events,
        })
    }

    /// todo check if comment still holds
    /// Transforms a [`ContractResult`] from a dry run into a [`Result`] type, containing
    /// details of the [`DispatchError`] if the dry run failed.
    #[allow(clippy::type_complexity)]
    fn contract_result_to_result<V>(
        &self,
        contract_result: ContractResult<V, E::Balance>,
    ) -> Result<ContractResult<V, E::Balance>, DryRunError<DispatchError>> {
        if let Err(error) = contract_result.result {
            let subxt_dispatch_err =
                self.runtime_dispatch_error_to_subxt_dispatch_error(&error);
            Err(DryRunError::<DispatchError> {
                error: subxt_dispatch_err,
            })
        } else {
            Ok(contract_result)
        }
    }

    /// Converts a `sp_runtime::DispatchError` into a `DispatchError` which contains error
    /// details.
    fn runtime_dispatch_error_to_subxt_dispatch_error(
        &self,
        dispatch_error: &sp_runtime::DispatchError,
    ) -> DispatchError {
        let dispatch_err_encoded = Encode::encode(&dispatch_error);
        DispatchError::decode_from(dispatch_err_encoded, self.api.client.metadata())
            .expect("failed to decode valid dispatch error")
    }

    /// Returns the URL of the running node.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Derives the Ethereum address from a keypair.
    // copied from `pallet-revive`
    fn derive_keypair_address(&self, signer: &Keypair) -> H160 {
        let account_id = <Keypair as subxt::tx::Signer<C>>::account_id(signer);
        let account_bytes = account_id.encode();
        crate::AccountIdMapper::to_address(account_bytes.as_ref())
    }

    /// Returns the original mapped `AccountId32` for a `H160`.
    ///
    /// Returns `None` if no mapping is found in the `pallet-revive` runtime
    /// storage; this is the case for e.g. contracts.
    async fn fetch_original_account(
        &self,
        addr: &H160,
    ) -> Result<Option<E::AccountId>, Error> {
        let original_account_entry = subxt::dynamic::storage(
            "Revive",
            "OriginalAccount",
            vec![Value::from_bytes(addr)],
        );
        let best_block = self.api.best_block().await;
        let raw_value = self
            .api
            .client
            .storage()
            .at(best_block)
            .fetch(&original_account_entry)
            .await
            .map_err(|err| {
                Error::Other(format!("Unable to fetch original account: {err:?}"))
            })?;
        Ok(match raw_value {
            Some(value) => {
                let raw_account_id = value.as_type::<[u8; 32]>().map_err(|err| {
                    Error::Decoding(format!("unable to deserialize AccountId: {err}"))
                })?;
                let account: E::AccountId = Decode::decode(&mut &raw_account_id[..])
                    .map_err(|err| {
                        Error::Decoding(format!("unable to decode AccountId: {err}"))
                    })?;
                Some(account)
            }
            None => None,
        })
    }

    /// Returns the `AccountId` for a `H160`.
    ///
    /// Queries runtime, returns fallback account if no result.
    pub async fn to_account_id(&self, addr: &H160) -> Result<E::AccountId, Error> {
        match self.fetch_original_account(addr).await? {
            Some(v) => Ok(v),
            None => {
                let fallback = to_fallback_account_id(addr);
                let account_id = E::AccountId::decode(&mut &fallback[..]).unwrap();
                Ok(account_id)
            }
        }
    }
}

/// Returns the fallback accountfor an `H160`.
fn to_fallback_account_id(address: &H160) -> [u8; 32] {
    let mut account_id = [0xEE; 32];
    account_id[..20].copy_from_slice(address.as_bytes());
    account_id
}

#[async_trait]
impl<C, E> ChainBackend for Client<C, E>
where
    C: subxt::Config + Send + Sync,
    C::AccountId: Clone
        + Debug
        + Send
        + Sync
        + core::fmt::Display
        + scale::Codec
        + From<sr25519::PublicKey>
        + serde::de::DeserializeOwned,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    C::Address: Send + Sync,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::Params:
        From<<DefaultExtrinsicParams<C> as ExtrinsicParams<C>>::Params>,
    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::Balance: Clone
        + Debug
        + Send
        + Sync
        + TryFrom<u128>
        + scale::HasCompact
        + serde::Serialize,
    E::EventRecord: Debug,
{
    type AccountId = E::AccountId;
    type Balance = E::Balance;
    type Error = Error;
    type EventLog = ExtrinsicEvents<C>;

    async fn create_and_fund_account(
        &mut self,
        origin: &Keypair,
        amount: Self::Balance,
    ) -> Keypair {
        let (_, phrase, _) =
            <sp_core::sr25519::Pair as sp_core::Pair>::generate_with_phrase(None);
        let phrase =
            subxt_signer::bip39::Mnemonic::parse(phrase).expect("valid phrase expected");
        let keypair = Keypair::from_phrase(&phrase, None).expect("valid phrase expected");
        let account_id = <Keypair as Signer<C>>::account_id(&keypair);
        let origin_account_id = origin.public_key().to_account_id();

        self.api
            .transfer_allow_death(origin, account_id.clone(), amount)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "transfer from {origin_account_id} to {account_id} failed with {err:?}"
                )
            });

        log_info(&format!(
            "transfer from {origin_account_id} to {account_id} succeeded",
        ));

        keypair
    }

    async fn free_balance(
        &mut self,
        account: Self::AccountId,
    ) -> Result<Self::Balance, Self::Error> {
        let account_addr = subxt::dynamic::storage(
            "System",
            "Account",
            vec![
                // Something that encodes to an AccountId32 is what we need for the map
                // key here:
                Value::from_bytes(&account),
            ],
        );

        let best_block = self.api.best_block().await;

        let account = self
            .api
            .client
            .storage()
            .at(best_block)
            .fetch_or_default(&account_addr)
            .await
            .unwrap_or_else(|err| {
                panic!("unable to fetch balance: {err:?}");
            })
            .to_value()
            .unwrap_or_else(|err| {
                panic!("unable to decode account info: {err:?}");
            });

        let account_data = get_composite_field_value(&account, "data")?;
        let balance = get_composite_field_value(account_data, "free")?;
        let balance = balance.as_u128().ok_or_else(|| {
            Error::Balance(format!("{balance:?} should convert to u128"))
        })?;
        let balance = E::Balance::try_from(balance).map_err(|_| {
            Error::Balance(format!("{balance:?} failed to convert from u128"))
        })?;

        log_info(&format!("balance of contract {account:?} is {balance:?}"));
        Ok(balance)
    }

    async fn runtime_call<'a>(
        &mut self,
        origin: &Keypair,
        pallet_name: &'a str,
        call_name: &'a str,
        call_data: Vec<Value>,
    ) -> Result<Self::EventLog, Self::Error> {
        let (tx_events, trace) = self
            .api
            .runtime_call(origin, pallet_name, call_name, call_data)
            .await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for `runtime_call` failed: {dispatch_error} {trace:?}"
                ));
                return Err(Error::CallExtrinsic(dispatch_error, trace))
            }
        }

        Ok(tx_events)
    }

    async fn transfer_allow_death(
        &mut self,
        origin: &Keypair,
        dest: Self::AccountId,
        value: Self::Balance,
    ) -> Result<(), Self::Error> {
        let dest = dest.encode();
        let dest: C::AccountId = Decode::decode(&mut &dest[..]).unwrap();
        self.api
            .transfer_allow_death(origin, dest, value)
            .await
            .map_err(|err| Error::Balance(format!("{err:?}")))
    }
}

#[async_trait]
impl<C, E> BuilderClient<E> for Client<C, E>
where
    C: subxt::Config + Send + Sync,
    C::AccountId: Clone
        + Debug
        + Send
        + Sync
        + core::fmt::Display
        + scale::Codec
        + From<sr25519::PublicKey>
        + From<[u8; 32]>
        + serde::de::DeserializeOwned,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    C::Address: Send + Sync,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::Params:
        From<<DefaultExtrinsicParams<C> as ExtrinsicParams<C>>::Params>,
    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::EventRecord: Debug,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    H256: Debug + Send + Sync + scale::Encode,
{
    fn load_code(&self, contract_name: &str) -> Vec<u8> {
        self.contracts.load_code(contract_name)
    }

    async fn bare_instantiate<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Abi> + Clone,
        R,
        Abi: Send + Sync + Clone,
    >(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let data = constructor_exec_input(constructor.clone());
        let ret = self
            .raw_instantiate(code, caller, data, value, gas_limit, storage_deposit_limit)
            .await?;
        Ok(ret)
    }

    async fn raw_instantiate(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        constructor: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let storage_deposit_limit = deposit_limit_to_balance::<E>(storage_deposit_limit);
        let (events, trace) = self
            .api
            .instantiate_with_code(
                value,
                gas_limit.into(),
                storage_deposit_limit,
                code.clone(),
                constructor.clone(),
                salt(),
                caller,
            )
            .await;

        let mut addr = None;
        for evt in events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });
            if let Some(instantiated) = evt
                .as_event::<ContractInstantiatedEvent>()
                .unwrap_or_else(|err| {
                    panic!("event conversion to `Instantiated` failed: {err:?}");
                })
            {
                log_info(&format!(
                    "contract was instantiated at {:?}",
                    instantiated.contract
                ));
                addr = Some(instantiated.contract);

                // We can't `break` here, we need to assign the account id from the
                // last `ContractInstantiatedEvent`, in case the contract instantiates
                // multiple accounts as part of its constructor!
            } else if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for instantiate failed: {dispatch_error} {trace:?}"
                ));
                return Err(Error::InstantiateExtrinsic(dispatch_error, trace))
            }
        }
        let addr = addr.expect("cannot extract contract address from events");
        let mut account_id = [0xEE; 32];
        account_id[..20].copy_from_slice(addr.as_bytes());

        Ok(BareInstantiationResult {
            // The `account_id` must exist at this point. If the instantiation fails
            // the dry-run must already return that.
            addr,
            account_id: E::AccountId::decode(&mut &account_id[..]).unwrap(),
            events,
            trace,
            code_hash: H256(crate::client_utils::code_hash(&code[..])),
        })
    }

    async fn exec_instantiate(
        &mut self,
        signer: &Keypair,
        contract_name: &str,
        data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        self.raw_instantiate(
            code,
            signer,
            data,
            value,
            gas_limit,
            DepositLimit::Balance(storage_deposit_limit),
        )
        .await
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_instantiate_dry_run<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Abi> + Clone,
        R,
        Abi: Send + Sync + Clone,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E, Abi>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());
        self.raw_instantiate_dry_run(code, caller, data, value, storage_deposit_limit)
            .await
    }

    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn raw_instantiate_dry_run<Abi: Sync + Clone>(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E, Abi>, Self::Error> {
        // There's a side effect here!
        let _ = self.map_account(caller).await;

        let result = self
            .api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code,
                data,
                salt(),
                caller,
            )
            .await;

        log_info(&format!("instantiate dry run: {:?}", &result.result));
        let result = self
            .contract_result_to_result(result)
            .map_err(Error::InstantiateDryRun)?;

        /*
        // todo
        if let Ok(res) = result.result.clone() {
            if res.result.did_revert() {
                return Err(Self::Error::InstantiateDryRunReverted(DryRunRevert {
                    error: res.result.data,
                }));
            }
        }
         */

        Ok(result.into())
    }

    async fn bare_upload(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let ret = self
            .exec_upload(caller, code, storage_deposit_limit)
            .await?;
        log_info(&format!("contract stored with hash {:?}", ret.code_hash));
        Ok(ret)
    }

    async fn bare_remove_code(
        &mut self,
        caller: &Keypair,
        code_hash: H256,
    ) -> Result<Self::EventLog, Self::Error> {
        let (tx_events, trace) = self.api.remove_code(caller, code_hash).await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for remove code failed: {dispatch_error} {trace:?}"
                ));
                return Err(Error::RemoveCodeExtrinsic(dispatch_error, trace))
            }
        }

        Ok(tx_events)
    }

    async fn bare_call<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType, Abi>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<(Self::EventLog, Option<CallTrace>), Self::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone,
    {
        let addr = *message.clone().params().callee();
        let exec_input = message.clone().params().exec_input().encode();
        log_info(&format!("call: {exec_input:02X?}"));
        self.raw_call(
            addr,
            exec_input,
            value,
            gas_limit,
            storage_deposit_limit,
            caller,
        )
        .await
    }

    async fn raw_call(
        &mut self,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
        signer: &Keypair,
    ) -> Result<(Self::EventLog, Option<CallTrace>), Self::Error> {
        let (tx_events, trace) = self
            .api
            .call(
                dest,
                value,
                gas_limit.into(),
                deposit_limit_to_balance::<E>(storage_deposit_limit),
                input_data,
                signer,
            )
            .await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for `raw_call` failed: {dispatch_error} {trace:?}"
                ));
                return Err(Error::CallExtrinsic(dispatch_error, trace))
            }
        }

        Ok((tx_events, trace))
    }

    /// Dry-runs a bare call.
    ///
    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_call_dry_run<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType, Abi>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<CallDryRunResult<E, RetType, Abi>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone,
    {
        // There's a side effect here!
        let _ = self.map_account(caller).await;

        let dest = *message.clone().params().callee();
        let exec_input = message.clone().params().exec_input().encode();

        let (exec_result, trace) = self
            .api
            .call_dry_run(dest, exec_input, value, storage_deposit_limit, caller)
            .await;
        log_info(&format!("call dry run result: {:?}", &exec_result.result));

        let exec_result = self
            .contract_result_to_result(exec_result)
            .map_err(Error::CallDryRun)?;

        Ok(CallDryRunResult {
            exec_result,
            trace,
            _marker: Default::default(),
        })
    }

    async fn raw_call_dry_run<
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
        signer: &Keypair,
    ) -> Result<CallDryRunResult<E, RetType, Abi>, Self::Error> {
        let (exec_result, trace) = self
            .api
            .call_dry_run(dest, input_data, value, storage_deposit_limit, signer)
            .await;
        Ok(CallDryRunResult {
            exec_result,
            trace,
            _marker: Default::default(),
        })
    }

    /// Checks if `caller` was already mapped in `pallet-revive`. If not, it will do so
    /// and return the events associated with that transaction.
    async fn map_account(
        &mut self,
        caller: &Keypair,
    ) -> Result<Option<Self::EventLog>, Self::Error> {
        let addr = self.derive_keypair_address(caller);
        if self.fetch_original_account(&addr).await?.is_some() {
            return Ok(None);
        }
        let (tx_events, trace) = self.api.map_account(caller).await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for `map_account` failed: {dispatch_error} {trace:?}"
                ));
                return Err(Error::CallExtrinsic(dispatch_error, trace))
            }
        }

        Ok(Some(tx_events))
    }

    async fn to_account_id(&mut self, addr: &H160) -> Result<E::AccountId, Self::Error> {
        let contract_info_address =
            dynamic("Revive", "OriginalAccount", vec![Value::from_bytes(addr)]);
        let raw_value = self
            .api
            .client
            .storage()
            .at_latest()
            .await
            .map_err(|err| Error::Other(format!("failed to fetch latest: {err:?}")))?
            .fetch(&contract_info_address)
            .await
            .map_err(|err| {
                Error::Other(format!("failed to fetch account info: {err:?}"))
            })?;
        match raw_value {
            None => {
                // This typically happens when calling this function with a contract, for
                // which there is no `AccountId`.
                let fallback = to_fallback_account_id(addr);
                tracing::debug!(
                    "No address suffix was found in the node for H160 address {:?}, using fallback {:?}",
                    addr,
                    fallback
                );
                let account_id = E::AccountId::decode(&mut &fallback[..]).unwrap();
                Ok(account_id)
            }
            Some(raw_value) => {
                let raw_account_id = raw_value.as_type::<[u8; 32]>().expect("oh");
                let account: E::AccountId = Decode::decode(&mut &raw_account_id[..])
                    .map_err(|err| {
                        panic!("AccountId from `[u8; 32]` deserialization error: {}", err)
                    })?;
                Ok(account)
            }
        }
    }
}

impl<C, E> ContractsBackend<E> for Client<C, E>
where
    C: subxt::Config + Send + Sync,
    C::AccountId: Clone
        + Debug
        + Send
        + Sync
        + core::fmt::Display
        + scale::Codec
        + From<sr25519::PublicKey>
        + serde::de::DeserializeOwned,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    C::Address: Send + Sync,
    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    H256: Debug + Send + scale::Encode,
{
    type Error = Error;
    type EventLog = ExtrinsicEvents<C>;
}

impl<C, E> E2EBackend<E> for Client<C, E>
where
    C: subxt::Config + Send + Sync,
    C::AccountId: Clone
        + Debug
        + Send
        + Sync
        + core::fmt::Display
        + scale::Codec
        + From<sr25519::PublicKey>
        + From<[u8; 32]>
        + serde::de::DeserializeOwned,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    C::Address: Send + Sync,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::Params:
        From<<DefaultExtrinsicParams<C> as ExtrinsicParams<C>>::Params>,
    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::EventRecord: Debug,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    H256: Debug + Send + Sync + scale::Encode,
{
}

/// Try to extract the given field from a dynamic [`Value`].
///
/// Returns `Err` if:
///   - The value is not a [`Value::Composite`] with [`Composite::Named`] fields
///   - The value does not contain a field with the given name.
fn get_composite_field_value<'a, T>(
    value: &'a Value<T>,
    field_name: &str,
) -> Result<&'a Value<T>, Error> {
    if let ValueDef::Composite(Composite::Named(fields)) = &value.value {
        let (_, field) = fields
            .iter()
            .find(|(name, _)| name == field_name)
            .ok_or_else(|| {
                Error::Balance(format!("No field named '{field_name}' found"))
            })?;
        Ok(field)
    } else {
        Err(Error::Balance(
            "Expected a composite type with named fields".into(),
        ))
    }
}

/// Returns true if the give event is System::Extrinsic failed.
fn is_extrinsic_failed_event<C: subxt::Config>(event: &EventDetails<C>) -> bool {
    event.pallet_name() == "System" && event.variant_name() == "ExtrinsicFailed"
}

impl<E: Environment, V, C: subxt::Config, Abi> CallResult<E, V, ExtrinsicEvents<C>, Abi> {
    /// Returns true if the specified event was triggered by the call.
    pub fn contains_event(&self, pallet_name: &str, variant_name: &str) -> bool {
        self.events.iter().any(|event| {
            let event = event.expect("unable to extract event");
            tracing::debug!(
                "found event with pallet: {:?}, variant: {:?}",
                event.pallet_name(),
                event.variant_name()
            );
            event.pallet_name() == pallet_name && event.variant_name() == variant_name
        })
    }

    /// Returns all the `ContractEmitted` events emitted by the contract.
    #[allow(clippy::result_large_err)] // todo
    pub fn contract_emitted_events(
        &self,
    ) -> Result<Vec<EventWithTopics<events::ContractEmitted>>, subxt::Error>
    where
        HashFor<C>: Into<sp_core::H256>,
    {
        let mut events_with_topics = Vec::new();
        for event in self.events.iter() {
            let event = event?;
            if let Some(decoded_event) = event.as_event::<events::ContractEmitted>()? {
                let topics = decoded_event.topics.clone();
                let event_with_topics = EventWithTopics {
                    event: decoded_event,
                    topics,
                };
                events_with_topics.push(event_with_topics);
            }
        }
        Ok(events_with_topics)
    }
}
