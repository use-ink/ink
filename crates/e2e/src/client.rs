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
};
use crate::contract_results::{
    CallDryRunResult,
    CallResult,
    InstantiationResult,
    UploadResult,
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
use sp_core::Pair;
#[cfg(feature = "std")]
use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::PathBuf,
};

use crate::events;
use subxt::{
    blocks::ExtrinsicEvents,
    config::ExtrinsicParams,
    events::EventDetails,
    ext::scale_value::{
        Composite,
        Value,
        ValueDef,
    },
};
use subxt_signer::sr25519::Keypair;

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
    contracts: BTreeMap<String, PathBuf>,
}

impl<C, E> Client<C, E>
where
    C: subxt::Config,
    C::AccountId: From<sp_runtime::AccountId32>
        + scale::Codec
        + serde::de::DeserializeOwned
        + Debug,
    C::Signature: From<sr25519::Signature>,
    <C::ExtrinsicParams as ExtrinsicParams<C::Index, C::Hash>>::OtherParams: Default,

    E: Environment,
    E::AccountId: Debug,
    E::Balance: Debug + scale::HasCompact + serde::Serialize,
    E::Hash: Debug + scale::Encode,
{
    /// Creates a new [`Client`] instance using a `subxt` client.
    pub async fn new(
        client: subxt::OnlineClient<C>,
        contracts: impl IntoIterator<Item = &str>,
    ) -> Self {
        let contracts = contracts
            .into_iter()
            .map(|path| {
                let wasm_path = PathBuf::from(path);
                let contract_name = wasm_path.file_stem().unwrap_or_else(|| {
                    panic!("Invalid contract wasm path '{}'", wasm_path.display(),)
                });
                (contract_name.to_string_lossy().to_string(), wasm_path)
            })
            .collect();

        Self {
            api: ContractsApi::new(client).await,
            contracts,
        }
    }

    /// Generate a new keypair and fund with the given amount from the origin account.
    ///
    /// Because many tests may execute this in parallel, transfers may fail due to a race
    /// condition with account indices. Therefore this will reattempt transfers a
    /// number of times.
    pub async fn create_and_fund_account(
        &self,
        origin: &Keypair,
        amount: E::Balance,
    ) -> Keypair
    where
        E::Balance: Clone,
        C::AccountId: Clone + core::fmt::Display + Debug,
        C::AccountId: From<[u8; 32]>,
    {
        let (_, phrase, _) = <sr25519::Pair as Pair>::generate_with_phrase(None);
        let phrase = subxt_signer::bip39::Mnemonic::parse(phrase).expect("valid phrase expected");
        let keypair = subxt_signer::sr25519::Keypair::from_phrase(&phrase, None).expect("valid phrase expected");
        let account_id = keypair.public_key().0.into();

        self.api
            .try_transfer_balance(origin, account_id.clone(), amount)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "transfer from {} to {} failed with {:?}",
                    origin.account_id(),
                    account_id,
                    err
                )
            });

        log_info(&format!(
            "transfer from {} to {} succeeded",
            origin.account_id(),
            account_id,
        ));

        keypair
    }

    /// This function extracts the metadata of the contract at the file path
    /// `target/ink/$contract_name.contract`.
    ///
    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// Calling this function multiple times is idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    pub async fn instantiate<Contract, Args, R>(
        &mut self,
        contract_name: &str,
        signer: &Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<E, ExtrinsicEvents<C>>, Error<E>>
    where
        Args: scale::Encode,
    {
        let code = self.load_code(contract_name);
        let ret = self
            .exec_instantiate::<Contract, Args, R>(
                signer,
                code,
                constructor,
                value,
                storage_deposit_limit,
            )
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }

    /// Dry run contract instantiation using the given constructor.
    pub async fn instantiate_dry_run<Contract, Args, R>(
        &mut self,
        contract_name: &str,
        signer: &Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()>
    where
        Args: scale::Encode,
    {
        let code = self.load_code(contract_name);
        let data = constructor_exec_input(constructor);

        let salt = Self::salt();
        self.api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code,
                data,
                salt,
                signer,
            )
            .await
    }

    /// Load the Wasm code for the given contract.
    fn load_code(&self, contract: &str) -> Vec<u8> {
        let wasm_path = self
            .contracts
            .get(&contract.replace('-', "_"))
            .unwrap_or_else(||
                panic!(
                    "Unknown contract {contract}. Available contracts: {:?}.\n\
                     For a contract to be built, add it as a dependency to the `Cargo.toml`, or add \
                     the manifest path to `#[ink_e2e::test(additional_contracts = ..)]`",
                    self.contracts.keys()
                )
            );
        let code = std::fs::read(wasm_path).unwrap_or_else(|err| {
            panic!("Error loading '{}': {:?}", wasm_path.display(), err)
        });
        log_info(&format!("{:?} has {} KiB", contract, code.len() / 1024));
        code
    }

    /// Executes an `instantiate_with_code` call and captures the resulting events.
    async fn exec_instantiate<Contract, Args, R>(
        &mut self,
        signer: &Keypair,
        code: Vec<u8>,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<E, ExtrinsicEvents<C>>, Error<E>>
    where
        Args: scale::Encode,
    {
        let salt = Self::salt();
        let data = constructor_exec_input(constructor);

        // dry run the instantiate to calculate the gas limit
        let dry_run = self
            .api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code.clone(),
                data.clone(),
                salt.clone(),
                signer,
            )
            .await;
        log_info(&format!(
            "instantiate dry run debug message: {:?}",
            String::from_utf8_lossy(&dry_run.debug_message)
        ));
        log_info(&format!("instantiate dry run result: {:?}", dry_run.result));
        if dry_run.result.is_err() {
            return Err(Error::<E>::InstantiateDryRun(dry_run))
        }

        let tx_events = self
            .api
            .instantiate_with_code(
                value,
                dry_run.gas_required.into(),
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

        Ok(InstantiationResult {
            dry_run,
            // The `account_id` must exist at this point. If the instantiation fails
            // the dry-run must already return that.
            account_id,
            events: tx_events,
        })
    }

    /// Generate a unique salt based on the system time.
    fn salt() -> Vec<u8> {
        use funty::Fundamental as _;

        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|err| panic!("unable to get unix time: {err}"))
            .as_millis()
            .as_u128()
            .to_le_bytes()
            .to_vec()
    }

    /// This function extracts the Wasm of the contract for the specified contract.
    ///
    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// Calling this function multiple times is idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    pub async fn upload(
        &mut self,
        contract_name: &str,
        signer: &Keypair,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, ExtrinsicEvents<C>>, Error<E>> {
        let code = self.load_code(contract_name);
        let ret = self
            .exec_upload(signer, code, storage_deposit_limit)
            .await?;
        log_info(&format!("contract stored with hash {:?}", ret.code_hash));
        Ok(ret)
    }

    /// Executes an `upload` call and captures the resulting events.
    pub async fn exec_upload(
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

    /// Executes a `call` for the contract at `account_id`.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn call<Args, RetType>(
        &mut self,
        signer: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<E, RetType, ExtrinsicEvents<C>>, Error<E>>
    where
        Args: scale::Encode,
        RetType: scale::Decode,
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let account_id = message.clone().params().callee().clone();
        let exec_input = scale::Encode::encode(message.clone().params().exec_input());
        log_info(&format!("call: {:02X?}", exec_input));

        let dry_run = self.call_dry_run(signer, message, value, None).await;

        if dry_run.exec_result.result.is_err() {
            return Err(Error::<E>::CallDryRun(dry_run.exec_result))
        }

        let tx_events = self
            .api
            .call(
                subxt::utils::MultiAddress::Id(account_id.clone()),
                value,
                dry_run.exec_result.gas_required.into(),
                storage_deposit_limit,
                exec_input,
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
                    subxt::error::DispatchError::decode_from(evt.field_bytes(), metadata)
                        .map_err(|e| Error::<E>::Decoding(e.to_string()))?;
                log_error(&format!("extrinsic for call failed: {dispatch_error}"));
                return Err(Error::<E>::CallExtrinsic(dispatch_error))
            }
        }

        Ok(CallResult {
            dry_run,
            events: tx_events,
        })
    }

    /// Executes a runtime call `call_name` for the `pallet_name`.
    /// The `call_data` is a `Vec<Value>`
    ///
    /// Note:
    /// - `pallet_name` must be in camel case, for example `Balances`.
    /// - `call_name` must be snake case, for example `force_transfer`.
    /// - `call_data` is a `Vec<subxt::dynamic::Value>` that holds a representation of
    ///   some value.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn runtime_call<'a>(
        &mut self,
        signer: &Keypair,
        pallet_name: &'a str,
        call_name: &'a str,
        call_data: Vec<Value>,
    ) -> Result<ExtrinsicEvents<C>, Error<E>> {
        let tx_events = self
            .api
            .runtime_call(signer, pallet_name, call_name, call_data)
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

    /// Executes a dry-run `call`.
    ///
    /// Returns the result of the dry run, together with the decoded return value of the
    /// invoked message.
    pub async fn call_dry_run<Args, RetType>(
        &mut self,
        signer: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CallDryRunResult<E, RetType>
    where
        Args: scale::Encode,
        RetType: scale::Decode,
        C::AccountId: From<[u8; 32]>,
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        let dest = message.clone().params().callee().clone();
        let exec_input = scale::Encode::encode(message.clone().params().exec_input());

        let exec_result = self
            .api
            .call_dry_run(
                signer.public_key().0.into(),
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

    /// Returns the balance of `account_id`.
    pub async fn balance(&self, account_id: E::AccountId) -> Result<E::Balance, Error<E>>
    where
        E::Balance: TryFrom<u128>,
    {
        let account_addr = subxt::dynamic::storage(
            "System",
            "Account",
            vec![
                // Something that encodes to an AccountId32 is what we need for the map
                // key here:
                Value::from_bytes(&account_id),
            ],
        );

        let account = self
            .api
            .client
            .storage()
            .at_latest()
            .await
            .unwrap_or_else(|err| {
                panic!("unable to fetch balance: {err:?}");
            })
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

        log_info(&format!(
            "balance of contract {account_id:?} is {balance:?}"
        ));
        Ok(balance)
    }
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
