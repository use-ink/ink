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
    builders::{
        constructor_exec_input,
        CreateBuilderPartial,
    },
    events::{
        CodeStoredEvent,
        ContractInstantiatedEvent,
        EventWithTopics,
    },
    log_error,
    log_info,
    sr25519,
    ContractInstantiateResult,
    ContractsApi,
    Keypair,
};
use crate::{
    backend::BuilderClient,
    contract_results::{
        BareInstantiationResult,
        CallDryRunResult,
        CallResult,
        InstantiationResult,
        UploadResult,
    },
};
use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
        },
        Call,
        ExecutionInput,
    },
    Environment,
};
use jsonrpsee::core::async_trait;
use scale::{
    Decode,
    Encode,
};
use sp_weights::Weight;
#[cfg(feature = "std")]
use std::fmt::Debug;
use std::path::PathBuf;

use crate::{
    backend::ChainBackend,
    client_utils::{
        salt,
        ContractsRegistry,
    },
    events,
    ContractsBackend,
    E2EBackend,
};
use subxt::{
    blocks::ExtrinsicEvents,
    config::ExtrinsicParams,
    events::EventDetails,
    ext::scale_value::{
        Composite,
        Value,
        ValueDef,
    },
    tx::Signer,
};

pub type Error<E> = crate::error::Error<
    <E as Environment>::AccountId,
    <E as Environment>::Balance,
    <E as Environment>::Hash,
    subxt::error::DispatchError,
>;

/// Represents an initialized contract message builder.
pub type CallBuilderFinal<E, Args, RetType> = ink_env::call::CallBuilder<
    E,
    Set<Call<E>>,
    Set<ExecutionInput<Args>>,
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
    api: ContractsApi<C, E>,
    contracts: ContractsRegistry,
}

impl<C, E> Client<C, E>
where
    C: subxt::Config,
    C::AccountId:
        From<sr25519::PublicKey> + scale::Codec + serde::de::DeserializeOwned + Debug,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::OtherParams: Default,

    E: Environment,
    E::AccountId: Debug,
    E::Balance: Debug + scale::HasCompact + serde::Serialize,
    E::Hash: Debug + scale::Encode,
{
    /// Creates a new [`Client`] instance using a `subxt` client.
    pub async fn new<P: Into<PathBuf>>(
        client: subxt::backend::rpc::RpcClient,
        contracts: impl IntoIterator<Item = P>,
    ) -> Result<Self, subxt::Error> {
        Ok(Self {
            api: ContractsApi::new(client).await?,
            contracts: ContractsRegistry::new(contracts),
        })
    }

    /// Executes an `instantiate_with_code` call and captures the resulting events.
    async fn exec_instantiate(
        &mut self,
        signer: &Keypair,
        code: Vec<u8>,
        data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<BareInstantiationResult<E, ExtrinsicEvents<C>>, Error<E>> {
        let salt = salt();

        let tx_events = self
            .api
            .instantiate_with_code(
                value,
                gas_limit.into(),
                storage_deposit_limit,
                code,
                data.clone(),
                salt,
                signer,
            )
            .await;

        let mut account_id = None;
        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if let Some(instantiated) = evt
                .as_event::<ContractInstantiatedEvent<E>>()
                .unwrap_or_else(|err| {
                    panic!("event conversion to `Instantiated` failed: {err:?}");
                })
            {
                log_info(&format!(
                    "contract was instantiated at {:?}",
                    instantiated.contract
                ));
                account_id = Some(instantiated.contract);

                // We can't `break` here, we need to assign the account id from the
                // last `ContractInstantiatedEvent`, in case the contract instantiates
                // multiple accounts as part of its constructor!
            } else if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::<E>::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for instantiate failed: {dispatch_error}"
                ));
                return Err(Error::<E>::InstantiateExtrinsic(dispatch_error))
            }
        }
        let account_id = account_id.expect("cannot extract `account_id` from events");

        Ok(BareInstantiationResult {
            // The `account_id` must exist at this point. If the instantiation fails
            // the dry-run must already return that.
            account_id,
            events: tx_events,
        })
    }

    /// Executes an `upload` call and captures the resulting events.
    async fn exec_upload(
        &mut self,
        signer: &Keypair,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, ExtrinsicEvents<C>>, Error<E>> {
        // dry run the instantiate to calculate the gas limit
        let dry_run = self
            .api
            .upload_dry_run(signer, code.clone(), storage_deposit_limit)
            .await;
        log_info(&format!("upload dry run: {dry_run:?}"));
        if dry_run.is_err() {
            return Err(Error::<E>::UploadDryRun(dry_run))
        }

        let tx_events = self.api.upload(signer, code, storage_deposit_limit).await;

        let mut hash = None;
        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if let Some(uploaded) =
                evt.as_event::<CodeStoredEvent<E>>().unwrap_or_else(|err| {
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
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::<E>::Decoding(e.to_string()))?;

                log_error(&format!("extrinsic for upload failed: {dispatch_error}"));
                return Err(Error::<E>::UploadExtrinsic(dispatch_error))
            }
        }

        // The `pallet-contracts` behavior is that if the code was already stored on the
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
    <C::ExtrinsicParams as ExtrinsicParams<C>>::OtherParams: Default + Send + Sync,
    <<C as subxt::Config>::ExtrinsicParams as ExtrinsicParams<
        <C as subxt::Config>::Hash,
    >>::OtherParams: Default + Send + Sync,
    <C as subxt::Config>::ExtrinsicParams:
        subxt::config::ExtrinsicParams<<C as subxt::Config>::Hash>,
    <C as subxt::Config>::Hash: subxt::Config,

    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::Balance: Clone
        + Debug
        + Send
        + Sync
        + TryFrom<u128>
        + scale::HasCompact
        + serde::Serialize,
{
    type AccountId = E::AccountId;
    type Balance = E::Balance;
    type Error = Error<E>;
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
            .try_transfer_balance(origin, account_id.clone(), amount)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "transfer from {} to {} failed with {:?}",
                    origin_account_id, account_id, err
                )
            });

        log_info(&format!(
            "transfer from {} to {} succeeded",
            origin_account_id, account_id,
        ));

        keypair
    }

    async fn balance(
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

        let account_data = get_composite_field_value::<_, E>(&account, "data")?;
        let balance = get_composite_field_value::<_, E>(account_data, "free")?;
        let balance = balance.as_u128().ok_or_else(|| {
            Error::<E>::Balance(format!("{balance:?} should convert to u128"))
        })?;
        let balance = E::Balance::try_from(balance).map_err(|_| {
            Error::<E>::Balance(format!("{balance:?} failed to convert from u128"))
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
        let tx_events = self
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
                        .map_err(|e| Error::<E>::Decoding(e.to_string()))?;

                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::<E>::CallExtrinsic(dispatch_error))
            }
        }

        Ok(tx_events)
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
        + serde::de::DeserializeOwned,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    C::Address: Send + Sync,
    <C::ExtrinsicParams as ExtrinsicParams<C>>::OtherParams: Default + Send + Sync,
    <<C as subxt::Config>::ExtrinsicParams as ExtrinsicParams<
        <C as subxt::Config>::Hash,
    >>::OtherParams: Default + Send + Sync,
    <C as subxt::Config>::ExtrinsicParams:
        subxt::config::ExtrinsicParams<<C as subxt::Config>::Hash>,
    <C as subxt::Config>::Hash: subxt::Config,

    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    E::Hash: Debug + Send + scale::Encode,
{
    async fn bare_instantiate<Contract: Clone, Args: Send + Sync + Encode + Clone, R>(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());
        let ret = self
            .exec_instantiate(caller, code, data, value, gas_limit, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }

    async fn bare_instantiate_dry_run<
        Contract: Clone,
        Args: Send + Sync + Encode + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());

        self.api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code,
                data,
                salt(),
                caller,
            )
            .await
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

    async fn bare_call<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<Self::EventLog, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let account_id = message.clone().params().callee().clone();
        let exec_input = Encode::encode(message.clone().params().exec_input());
        log_info(&format!("call: {:02X?}", exec_input));

        let tx_events = self
            .api
            .call(
                subxt::utils::MultiAddress::Id(account_id.clone()),
                value,
                gas_limit.into(),
                storage_deposit_limit,
                exec_input,
                caller,
            )
            .await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::<E>::Decoding(e.to_string()))?;
                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::<E>::CallExtrinsic(dispatch_error))
            }
        }

        Ok(tx_events)
    }

    async fn bare_call_dry_run<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CallDryRunResult<E, RetType>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let dest = message.clone().params().callee().clone();
        let exec_input = Encode::encode(message.clone().params().exec_input());

        let exec_result = self
            .api
            .call_dry_run(
                Signer::<C>::account_id(caller),
                dest,
                exec_input,
                value,
                storage_deposit_limit,
            )
            .await;
        log_info(&format!("call dry run: {:?}", &exec_result.result));
        log_info(&format!(
            "call dry run debug message: {}",
            String::from_utf8_lossy(&exec_result.debug_message)
        ));

        CallDryRunResult {
            exec_result,
            _marker: Default::default(),
        }
    }
    async fn instantiate_with_gas_margin<
        Contract: Clone,
        Args: Send + Sync + Encode + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        margin: Option<u64>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());

        let dry_run = self
            .api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code.clone(),
                data.clone(),
                salt(),
                caller,
            )
            .await;

        if dry_run.result.is_err() {
            return Err(Self::Error::InstantiateDryRun(dry_run))
        }

        let gas_limit = if let Some(m) = margin {
            dry_run.gas_required + (dry_run.gas_required / 100 * m)
        } else {
            dry_run.gas_required
        };

        let res = self
            .exec_instantiate(caller, code, data, value, gas_limit, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", res.account_id));

        Ok(InstantiationResult {
            account_id: res.account_id,
            dry_run,
            events: res.events,
        })
    }

    async fn instantiate_with_gas_limit<
        Contract: Clone,
        Args: Send + Sync + Encode + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<E, Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());

        let dry_run = self
            .api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code.clone(),
                data.clone(),
                salt(),
                caller,
            )
            .await;

        if dry_run.result.is_err() {
            return Err(Self::Error::InstantiateDryRun(dry_run))
        }

        let res = self
            .exec_instantiate(caller, code, data, value, gas_limit, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", res.account_id));

        Ok(InstantiationResult {
            account_id: res.account_id,
            dry_run,
            events: res.events,
        })
    }

    async fn call_with_gas_margin<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        margin: Option<u64>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<E, RetType, Self::EventLog>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let dry_run = self
            .bare_call_dry_run(caller, message, value, storage_deposit_limit)
            .await;

        if dry_run.is_err() {
            return Err(Self::Error::CallDryRun(dry_run.exec_result))
        }

        let gas_required: Weight = margin.map_or(dry_run.exec_result.gas_required, |m| {
            let gas = dry_run.exec_result.gas_required;
            gas + (gas / 100 * m)
        });
        let call_result = self
            .bare_call(caller, message, value, gas_required, storage_deposit_limit)
            .await?;
        Ok(CallResult {
            dry_run,
            events: call_result,
        })
    }

    async fn call_with_gas_limit<Args: Sync + Encode + Clone, RetType: Send + Decode>(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<E, RetType, Self::EventLog>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let dry_run = self
            .bare_call_dry_run(caller, message, value, storage_deposit_limit)
            .await;

        if dry_run.is_err() {
            return Err(Self::Error::CallDryRun(dry_run.exec_result))
        }

        let call_result = self
            .bare_call(caller, message, value, gas_limit, storage_deposit_limit)
            .await?;
        Ok(CallResult {
            dry_run,
            events: call_result,
        })
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
    <C::ExtrinsicParams as ExtrinsicParams<C::Hash>>::OtherParams: Default + Send + Sync,
    <<C as subxt::Config>::ExtrinsicParams as ExtrinsicParams<
        <C as subxt::Config>::Hash,
    >>::OtherParams: Default + Send + Sync,
    <C as subxt::Config>::ExtrinsicParams:
        subxt::config::ExtrinsicParams<<C as subxt::Config>::Hash>,
    <C as subxt::Config>::Hash: subxt::Config,

    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    E::Hash: Debug + Send + scale::Encode,
{
    type Error = Error<E>;
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
        + serde::de::DeserializeOwned,
    C::Address: From<sr25519::PublicKey>,
    C::Signature: From<sr25519::Signature>,
    C::Address: Send + Sync,
    <<C as subxt::Config>::ExtrinsicParams as subxt::config::ExtrinsicParams<C>>::OtherParams: Default + Send + Sync,
    <<C as subxt::Config>::ExtrinsicParams as ExtrinsicParams<
        <C as subxt::Config>::Hash,
    >>::OtherParams: Default + Send + Sync,
    <C as subxt::Config>::ExtrinsicParams:
        subxt::config::ExtrinsicParams<<C as subxt::Config>::Hash>,
    <C as subxt::Config>::Hash: subxt::Config,

    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    E::Hash: Debug + Send + scale::Encode,
{
}

/// Try to extract the given field from a dynamic [`Value`].
///
/// Returns `Err` if:
///   - The value is not a [`Value::Composite`] with [`Composite::Named`] fields
///   - The value does not contain a field with the given name.
fn get_composite_field_value<'a, T, E>(
    value: &'a Value<T>,
    field_name: &str,
) -> Result<&'a Value<T>, Error<E>>
where
    E: Environment,
    E::Balance: Debug,
{
    if let ValueDef::Composite(Composite::Named(fields)) = &value.value {
        let (_, field) = fields
            .iter()
            .find(|(name, _)| name == field_name)
            .ok_or_else(|| {
                Error::<E>::Balance(format!("No field named '{field_name}' found"))
            })?;
        Ok(field)
    } else {
        Err(Error::<E>::Balance(
            "Expected a composite type with named fields".into(),
        ))
    }
}

/// Returns true if the give event is System::Extrinsic failed.
fn is_extrinsic_failed_event<C: subxt::Config>(event: &EventDetails<C>) -> bool {
    event.pallet_name() == "System" && event.variant_name() == "ExtrinsicFailed"
}

impl<E: Environment, V, C: subxt::Config> CallResult<E, V, ExtrinsicEvents<C>> {
    /// Returns true if the specified event was triggered by the call.
    pub fn contains_event(&self, pallet_name: &str, variant_name: &str) -> bool {
        self.events.iter().any(|event| {
            let event = event.unwrap();
            event.pallet_name() == pallet_name && event.variant_name() == variant_name
        })
    }

    /// Returns all the `ContractEmitted` events emitted by the contract.
    pub fn contract_emitted_events(
        &self,
    ) -> Result<Vec<EventWithTopics<events::ContractEmitted<E>>>, subxt::Error>
    where
        C::Hash: Into<sp_core::H256>,
    {
        let mut events_with_topics = Vec::new();
        for event in self.events.iter() {
            let event = event?;
            if let Some(decoded_event) = event.as_event::<events::ContractEmitted<E>>()? {
                let event_with_topics = EventWithTopics {
                    event: decoded_event,
                    topics: event.topics().iter().cloned().map(Into::into).collect(),
                };
                events_with_topics.push(event_with_topics);
            }
        }
        Ok(events_with_topics)
    }
}
