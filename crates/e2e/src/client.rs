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
        Message,
    },
    log_error,
    log_info,
    sr25519,
    CodeUploadResult,
    ContractExecResult,
    ContractInstantiateResult,
    ContractsApi,
    Signer,
};
use ink_env::Environment;
use ink_primitives::MessageResult;
use pallet_contracts_primitives::ExecReturnValue;
use sp_core::Pair;
use std::{
    collections::BTreeMap,
    fmt::Debug,
    marker::PhantomData,
    path::PathBuf,
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
    tx::PairSigner,
};

/// Result of a contract instantiation.
pub struct InstantiationResult<C: subxt::Config, E: Environment> {
    /// The account id at which the contract was instantiated.
    pub account_id: E::AccountId,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: ContractInstantiateResult<C::AccountId, E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: ExtrinsicEvents<C>,
}

/// Result of a contract upload.
pub struct UploadResult<C: subxt::Config, E: Environment> {
    /// The hash with which the contract can be instantiated.
    pub code_hash: E::Hash,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: CodeUploadResult<E::Hash, E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: ExtrinsicEvents<C>,
}

/// We implement a custom `Debug` here, to avoid requiring the trait
/// bound `Debug` for `E`.
impl<C, E> Debug for UploadResult<C, E>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: Debug,
    <E as Environment>::Hash: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("UploadResult")
            .field("code_hash", &self.code_hash)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait
/// bound `Debug` for `E`.
impl<C, E> core::fmt::Debug for InstantiationResult<C, E>
where
    C: subxt::Config,
    C::AccountId: Debug,
    E: Environment,
    <E as Environment>::AccountId: Debug,
    <E as Environment>::Balance: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InstantiationResult")
            .field("account_id", &self.account_id)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract call.
pub struct CallResult<C: subxt::Config, E: Environment, V> {
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: CallDryRunResult<E, V>,
    /// Events that happened with the contract instantiation.
    pub events: ExtrinsicEvents<C>,
}

impl<C, E, V> CallResult<C, E, V>
where
    C: subxt::Config,
    E: Environment,
    V: scale::Decode,
{
    /// Returns the [`MessageResult`] from the execution of the dry-run message
    /// call.
    ///
    /// # Panics
    /// - if the dry-run message call failed to execute.
    /// - if message result cannot be decoded into the expected return value
    ///   type.
    pub fn message_result(&self) -> MessageResult<V> {
        self.dry_run.message_result()
    }

    /// Returns the decoded return value of the message from the dry-run.
    ///
    /// Panics if the value could not be decoded. The raw bytes can be accessed
    /// via [`CallResult::return_data`].
    pub fn return_value(self) -> V {
        self.dry_run.return_value()
    }

    /// Returns the return value as raw bytes of the message from the dry-run.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.dry_run.exec_return_value().data
    }

    /// Returns any debug message output by the contract decoded as UTF-8.
    pub fn debug_message(&self) -> String {
        self.dry_run.debug_message()
    }

    /// Returns true if the specified event was triggered by the call.
    pub fn contains_event(&self, pallet_name: &str, variant_name: &str) -> bool {
        self.events.iter().any(|event| {
            let event = event.unwrap();
            event.pallet_name() == pallet_name && event.variant_name() == variant_name
        })
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait
/// bound `Debug` for `E`.
// TODO(#xxx) Improve the `Debug` implementation.
impl<C, E, V> Debug for CallResult<C, E, V>
where
    C: subxt::Config + Debug,
    E: Environment + Debug,
    <E as Environment>::Balance: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("CallResult")
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of the dry run of a contract call.
#[derive(Debug)]
pub struct CallDryRunResult<E: Environment, V> {
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub exec_result: ContractExecResult<E::Balance>,
    _marker: PhantomData<V>,
}

impl<E, V> CallDryRunResult<E, V>
where
    E: Environment,
    V: scale::Decode,
{
    /// Returns true if the dry-run execution resulted in an error.
    pub fn is_err(&self) -> bool {
        self.exec_result.result.is_err()
    }

    /// Returns the [`ExecReturnValue`] resulting from the dry-run message call.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn exec_return_value(&self) -> &ExecReturnValue {
        self.exec_result
            .result
            .as_ref()
            .unwrap_or_else(|call_err| panic!("Call dry-run failed: {call_err:?}"))
    }

    /// Returns the [`MessageResult`] from the execution of the dry-run message
    /// call.
    ///
    /// # Panics
    /// - if the dry-run message call failed to execute.
    /// - if message result cannot be decoded into the expected return value
    ///   type.
    pub fn message_result(&self) -> MessageResult<V> {
        let data = &self.exec_return_value().data;
        scale::Decode::decode(&mut data.as_ref()).unwrap_or_else(|env_err| {
            panic!(
                "Decoding dry run result to ink! message return type failed: {env_err}"
            )
        })
    }

    /// Returns the decoded return value of the message from the dry-run.
    ///
    /// Panics if the value could not be decoded. The raw bytes can be accessed
    /// via [`CallResult::return_data`].
    pub fn return_value(self) -> V {
        self.message_result()
            .unwrap_or_else(|lang_err| {
                panic!(
                    "Encountered a `LangError` while decoding dry run result to ink! message: {lang_err:?}"
                )
            })
    }

    /// Returns the return value as raw bytes of the message from the dry-run.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.exec_return_value().data
    }

    /// Returns any debug message output by the contract decoded as UTF-8.
    pub fn debug_message(&self) -> String {
        String::from_utf8_lossy(&self.exec_result.debug_message).into()
    }
}

/// An error occurred while interacting with the Substrate node.
///
/// We only convey errors here that are caused by the contract's
/// testing logic. For anything concerning the node (like inability
/// to communicate with it, fetch the nonce, account info, etc.) we
/// panic.
pub enum Error<C, E>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: core::fmt::Debug,
{
    /// No contract with the given name found in scope.
    ContractNotFound(String),
    /// The `instantiate_with_code` dry run failed.
    InstantiateDryRun(ContractInstantiateResult<C::AccountId, E::Balance>),
    /// The `instantiate_with_code` extrinsic failed.
    InstantiateExtrinsic(subxt::error::DispatchError),
    /// The `upload` dry run failed.
    UploadDryRun(CodeUploadResult<E::Hash, E::Balance>),
    /// The `upload` extrinsic failed.
    UploadExtrinsic(subxt::error::DispatchError),
    /// The `call` dry run failed.
    CallDryRun(ContractExecResult<E::Balance>),
    /// The `call` extrinsic failed.
    CallExtrinsic(subxt::error::DispatchError),
    /// Error fetching account balance.
    Balance(String),
}

// We implement a custom `Debug` here, as to avoid requiring the trait
// bound `Debug` for `C`.
// TODO(#xxx) Improve the Debug implementations below to also output `_`.
impl<C, E> core::fmt::Debug for Error<C, E>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self {
            Error::ContractNotFound(name) => {
                f.write_str(&format!("ContractNotFound: {name}"))
            }
            Error::InstantiateDryRun(res) => {
                f.write_str(&format!(
                    "InstantiateDryRun: {}",
                    &String::from_utf8_lossy(&res.debug_message)
                ))
            }
            Error::InstantiateExtrinsic(_) => f.write_str("InstantiateExtrinsic"),
            Error::UploadDryRun(_) => f.write_str("UploadDryRun"),
            Error::UploadExtrinsic(_) => f.write_str("UploadExtrinsic"),
            Error::CallDryRun(_) => f.write_str("CallDryRun"),
            Error::CallExtrinsic(_) => f.write_str("CallExtrinsic"),
            Error::Balance(msg) => write!(f, "Balance: {msg}"),
        }
    }
}

/// A contract was successfully instantiated.
#[derive(Debug, scale::Decode, scale::Encode)]
struct ContractInstantiatedEvent<E: Environment> {
    /// Account id of the deployer.
    pub deployer: E::AccountId,
    /// Account id where the contract was instantiated to.
    pub contract: E::AccountId,
}

impl<E> subxt::events::StaticEvent for ContractInstantiatedEvent<E>
where
    E: Environment,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "Instantiated";
}

/// Code with the specified hash has been stored.
#[derive(Debug, scale::Decode, scale::Encode)]
struct CodeStoredEvent<E: Environment> {
    /// Hash under which the contract code was stored.
    pub code_hash: E::Hash,
}

impl<E> subxt::events::StaticEvent for CodeStoredEvent<E>
where
    E: Environment,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "CodeStored";
}

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
    /// Because many tests may execute this in parallel, transfers may fail due to a race condition
    /// with account indices. Therefore this will reattempt transfers a number of times.
    pub async fn create_and_fund_account(
        &self,
        origin: &Signer<C>,
        amount: E::Balance,
    ) -> Signer<C>
    where
        E::Balance: Clone,
        C::AccountId: Clone + core::fmt::Display + Debug,
        C::AccountId: From<sp_core::crypto::AccountId32>,
    {
        let (pair, _, _) = <sr25519::Pair as Pair>::generate_with_phrase(None);
        let pair_signer = PairSigner::<C, _>::new(pair);
        let account_id = pair_signer.account_id().to_owned();

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

        pair_signer
    }

    /// This function extracts the metadata of the contract at the file path
    /// `target/ink/$contract_name.contract`.
    ///
    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// Calling this function multiple times is idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    pub async fn instantiate<ContractRef, Args, R>(
        &mut self,
        contract_name: &str,
        signer: &Signer<C>,
        constructor: CreateBuilderPartial<E, ContractRef, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    where
        Args: scale::Encode,
    {
        let code = self.load_code(contract_name);
        let ret = self
            .exec_instantiate(signer, code, constructor, value, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }

    /// Dry run contract instantiation using the given constructor.
    pub async fn instantiate_dry_run<ContractRef, Args, R>(
        &mut self,
        contract_name: &str,
        signer: &Signer<C>,
        constructor: CreateBuilderPartial<E, ContractRef, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<C::AccountId, E::Balance>
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
    async fn exec_instantiate<ContractRef, Args, R>(
        &mut self,
        signer: &Signer<C>,
        code: Vec<u8>,
        constructor: CreateBuilderPartial<E, ContractRef, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
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
            return Err(Error::InstantiateDryRun(dry_run))
        }

        let tx_events = self
            .api
            .instantiate_with_code(
                value,
                dry_run.gas_required,
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
                let dispatch_error = subxt::error::DispatchError::decode_from(
                    evt.field_bytes(),
                    &metadata,
                );
                log_error(&format!(
                    "extrinsic for instantiate failed: {dispatch_error:?}"
                ));
                return Err(Error::InstantiateExtrinsic(dispatch_error))
            }
        }

        Ok(InstantiationResult {
            dry_run,
            // The `account_id` must exist at this point. If the instantiation fails
            // the dry-run must already return that.
            account_id: account_id.expect("cannot extract `account_id` from events"),
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
        signer: &Signer<C>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<C, E>, Error<C, E>> {
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
        signer: &Signer<C>,
        code: Vec<u8>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<C, E>, Error<C, E>> {
        // dry run the instantiate to calculate the gas limit
        let dry_run = self
            .api
            .upload_dry_run(signer, code.clone(), storage_deposit_limit)
            .await;
        log_info(&format!("upload dry run: {dry_run:?}"));
        if dry_run.is_err() {
            return Err(Error::UploadDryRun(dry_run))
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
                let dispatch_error = subxt::error::DispatchError::decode_from(
                    evt.field_bytes(),
                    &metadata,
                );
                log_error(&format!("extrinsic for upload failed: {dispatch_error:?}"));
                return Err(Error::UploadExtrinsic(dispatch_error))
            }
        }

        // The `pallet-contracts` behavior is that if the code was already stored on the
        // chain we won't get an event with the hash, but the extrinsic will still succeed.
        // We then don't error (`cargo-contract` would), but instead return the hash from
        // the dry-run.
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
    pub async fn call<RetType>(
        &mut self,
        signer: &Signer<C>,
        message: Message<E, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<C, E, RetType>, Error<C, E>>
    where
        RetType: scale::Decode,
    {
        log_info(&format!("call: {:02X?}", message.exec_input()));

        let dry_run = self.call_dry_run(signer, &message, value, None).await;

        if dry_run.exec_result.result.is_err() {
            return Err(Error::CallDryRun(dry_run.exec_result))
        }

        let tx_events = self
            .api
            .call(
                sp_runtime::MultiAddress::Id(message.account_id().clone()),
                value,
                dry_run.exec_result.gas_required,
                storage_deposit_limit,
                message.exec_input().to_vec(),
                signer,
            )
            .await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {err:?}");
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error = subxt::error::DispatchError::decode_from(
                    evt.field_bytes(),
                    &metadata,
                );
                log_error(&format!("extrinsic for call failed: {dispatch_error:?}"));
                return Err(Error::CallExtrinsic(dispatch_error))
            }
        }

        Ok(CallResult {
            dry_run,
            events: tx_events,
        })
    }

    /// Executes a dry-run `call`.
    ///
    /// Returns the result of the dry run, together with the decoded return value of the invoked
    /// message.
    pub async fn call_dry_run<RetType>(
        &mut self,
        signer: &Signer<C>,
        message: &Message<E, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CallDryRunResult<E, RetType>
    where
        RetType: scale::Decode,
    {
        let exec_result = self
            .api
            .call_dry_run(
                Signer::account_id(signer).clone(),
                message,
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
    pub async fn balance(
        &self,
        account_id: E::AccountId,
    ) -> Result<E::Balance, Error<C, E>>
    where
        E::Balance: TryFrom<u128>,
    {
        let account_addr = subxt::dynamic::storage(
            "System",
            "Account",
            vec![
                // Something that encodes to an AccountId32 is what we need for the map key here:
                Value::from_bytes(&account_id),
            ],
        );

        let account = self
            .api
            .client
            .storage()
            .at(None)
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

        let account_data = get_composite_field_value(&account, "data")?;
        let balance = get_composite_field_value(account_data, "free")?;
        let balance = balance.as_u128().ok_or_else(|| {
            Error::Balance(format!("{balance:?} should convert to u128"))
        })?;
        let balance = E::Balance::try_from(balance).map_err(|_| {
            Error::Balance(format!("{balance:?} failed to convert from u128"))
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
fn get_composite_field_value<'a, T, C, E>(
    value: &'a Value<T>,
    field_name: &str,
) -> Result<&'a Value<T>, Error<C, E>>
where
    C: subxt::Config,
    E: Environment,
    E::Balance: Debug,
{
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
fn is_extrinsic_failed_event(event: &EventDetails) -> bool {
    event.pallet_name() == "System" && event.variant_name() == "ExtrinsicFailed"
}
