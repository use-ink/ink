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

use ink::H160;
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
use ink_primitives::{
    abi::{
        AbiDecodeWith,
        AbiEncodeWith,
        Ink,
    },
    types::AccountIdMapper,
    DepositLimit,
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
    tx::Signer,
};

use super::{
    builders::{
        constructor_exec_input,
        CreateBuilderPartial,
    },
    deposit_limit_to_balance,
    events::{
        CodeStoredEvent,
        EventWithTopics,
    },
    log_error,
    log_info,
    sr25519,
    InstantiateDryRunResult,
    Keypair,
    ReviveApi,
    H256,
};
use crate::{
    backend::{
        BuilderClient,
        ChainBackend,
    },
    client_utils::{
        salt,
        ContractsRegistry,
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
    ContractsBackend,
    E2EBackend,
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
    // TODO (@peterwht): make private once call builder supports RLP
    pub api: ReviveApi<C, E>,
    pub contracts: ContractsRegistry,
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

    // TODO (@peterwht): private after call builder supports RLP
    /// Executes an `instantiate_with_code` call and captures the resulting events.
    pub async fn exec_instantiate(
        &mut self,
        signer: &Keypair,
        code: Vec<u8>,
        data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<BareInstantiationResult<ExtrinsicEvents<C>>, Error> {
        let salt = salt();
        // todo remove assert once salt() returns no more option
        assert!(salt.is_some());
        let (events, trace) = self
            .api
            .instantiate_with_code(
                value,
                gas_limit.into(),
                storage_deposit_limit,
                code.clone(),
                data.clone(),
                salt,
                signer,
            )
            .await;

        for evt in events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });
            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!(
                    "extrinsic for instantiate failed: {dispatch_error}"
                ));
                return Err(Error::InstantiateExtrinsic(dispatch_error))
            }
        }

        let deployer = self.derive_keypair_address(signer);
        let addr = pallet_revive::create2(
            &deployer,
            &code[..],
            &data[..],
            &salt.expect("todo make salt() return no option, but value"),
        );

        Ok(BareInstantiationResult {
            // The `account_id` must exist at this point. If the instantiation fails
            // the dry-run must already return that.
            addr,
            events,
            trace,
            code_hash: H256(crate::client_utils::code_hash(&code[..])),
        })
    }

    /// Executes an `upload` call and captures the resulting events.
    async fn exec_upload(
        &mut self,
        signer: &Keypair,
        code: Vec<u8>,
        _storage_deposit_limit: E::Balance,
    ) -> Result<UploadResult<E, ExtrinsicEvents<C>>, Error> {
        // todo
        let storage_deposit_limit: E::Balance = unsafe {
            core::mem::transmute_copy::<u128, <E as Environment>::Balance>(&u128::MAX)
        };
        let dry_run = self
            .api
            .upload_dry_run(signer, code.clone(), storage_deposit_limit)
            .await;
        log_info(&format!("upload dry run: {dry_run:?}"));
        if let Err(err) = dry_run {
            let dispatch_err = self.runtime_dispatch_error_to_subxt_dispatch_error(&err);
            return Err(Error::UploadDryRun(dispatch_err))
        }

        let tx_events = self.api.upload(signer, code, storage_deposit_limit).await;

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

                log_error(&format!("extrinsic for upload failed: {dispatch_error}"));
                return Err(Error::UploadExtrinsic(dispatch_error))
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
        AccountIdMapper::to_address(account_bytes.as_ref())
    }

    /// Returns the original mapped `AccountId32` for a `H160`.
    async fn fetch_original_account(
        &self,
        addr: &H160,
    ) -> Result<Option<C::AccountId>, Error> {
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
                    Error::Decoding(format!("unable to deserialize AccountId: {}", err))
                })?;
                let account: C::AccountId = Decode::decode(&mut &raw_account_id[..])
                    .map_err(|err| {
                        Error::Decoding(format!("unable to decode AccountId: {}", err))
                    })?;
                Some(account)
            }
            None => None,
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
                        .map_err(|e| Error::Decoding(e.to_string()))?;

                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::CallExtrinsic(dispatch_error))
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
    <C::ExtrinsicParams as ExtrinsicParams<C>>::Params:
        From<<DefaultExtrinsicParams<C> as ExtrinsicParams<C>>::Params>,

    E: Environment,
    E::AccountId: Debug + Send + Sync,
    E::EventRecord: Debug,
    E::Balance:
        Clone + Debug + Send + Sync + From<u128> + scale::HasCompact + serde::Serialize,
    H256: Debug + Send + Sync + scale::Encode,
{
    async fn bare_instantiate<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Ink> + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<BareInstantiationResult<Self::EventLog>, Self::Error> {
        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());
        let storage_deposit_limit = deposit_limit_to_balance::<E>(storage_deposit_limit);
        let ret = self
            .exec_instantiate(caller, code, data, value, gas_limit, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.addr));
        Ok(ret)
    }

    async fn bare_instantiate_dry_run<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Ink> + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E>, Self::Error> {
        // todo beware side effect! this is wrong, we have to batch up the `map_account`
        // into the RPC dry run instead
        let _ = self.map_account(caller).await;

        let code = self.contracts.load_code(contract_name);
        let data = constructor_exec_input(constructor.clone());

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
        storage_deposit_limit: E::Balance,
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
        let tx_events = self.api.remove_code(caller, code_hash).await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                return Err(Error::RemoveCodeExtrinsic(dispatch_error))
            }
        }

        Ok(tx_events)
    }

    async fn bare_call<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + AbiDecodeWith<Abi>,
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
        log_info(&format!("call: {:02X?}", exec_input));

        let (tx_events, trace) = self
            .api
            .call(
                addr,
                value,
                gas_limit.into(),
                deposit_limit_to_balance::<E>(storage_deposit_limit),
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
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::CallExtrinsic(dispatch_error))
            }
        }

        Ok((tx_events, trace))
    }

    // todo is not really a `bare_call`
    async fn bare_call_dry_run<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + AbiDecodeWith<Abi>,
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
        // todo beware side effect! this is wrong, we have to batch up the `map_account`
        // into the RPC dry run instead
        let _ = self.map_account(caller).await;

        let dest = *message.clone().params().callee();
        let exec_input = message.clone().params().exec_input().encode();

        let (exec_result, trace) = self
            .api
            .call_dry_run(
                Signer::<C>::account_id(caller), /* todo this param is not necessary,
                                                  * because the last argument is the
                                                  * caller and this value can be
                                                  * created in the function */
                dest,
                exec_input,
                value,
                deposit_limit_to_balance::<E>(storage_deposit_limit),
                caller,
            )
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

    async fn map_account(&mut self, caller: &Keypair) -> Result<(), Self::Error> {
        let addr = self.derive_keypair_address(caller);
        if self.fetch_original_account(&addr).await?.is_some() {
            return Ok(());
        }
        let tx_events = self.api.map_account(caller).await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::CallExtrinsic(dispatch_error))
            }
        }

        // todo: Ok(tx_events)
        Ok(())
    }

    // todo not used anywhere
    // code is also not dry
    async fn map_account_dry_run(&mut self, caller: &Keypair) -> Result<(), Self::Error> {
        let tx_events = self.api.map_account(caller).await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error =
                    DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::Decoding(e.to_string()))?;
                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::CallExtrinsic(dispatch_error))
            }
        }

        Ok(())
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

impl<E: Environment, V, C: subxt::Config> CallResult<E, V, ExtrinsicEvents<C>> {
    /// Returns true if the specified event was triggered by the call.
    pub fn contains_event(&self, pallet_name: &str, variant_name: &str) -> bool {
        self.events.iter().any(|event| {
            let event = event.unwrap();
            eprintln!(
                "pallet: {:?}, variant: {:?}",
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
