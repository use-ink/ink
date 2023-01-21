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
    xts::{
        Call,
        InstantiateWithCode,
    },
    CodeUploadResult,
    ContractExecResult,
    ContractInstantiateResult,
    ContractsApi,
    Signer,
};
use contract_metadata::ContractMetadata;
use ink_env::Environment;
use ink_primitives::MessageResult;

use sp_runtime::traits::{
    IdentifyAccount,
    Verify,
};
use std::{
    collections::BTreeMap,
    fmt::Debug,
    path::Path,
};
use subxt::{
    blocks::ExtrinsicEvents,
    events::EventDetails,
    ext::{
        bitvec::macros::internal::funty::Fundamental,
        scale_value::{
            Composite,
            Value,
            ValueDef,
        },
    },
    tx::ExtrinsicParams,
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
    pub dry_run: ContractExecResult<E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: ExtrinsicEvents<C>,
    /// Contains the result of decoding the return value of the called
    /// function.
    pub value: Result<MessageResult<V>, scale::Error>,
    /// Returns the bytes of the encoded dry-run return value.
    pub data: Vec<u8>,
}

impl<C, E, V> CallResult<C, E, V>
where
    C: subxt::Config,
    E: Environment,
{
    /// Returns the decoded return value of the message from the dry-run.
    ///
    /// Panics if the value could not be decoded. The raw bytes can be accessed
    /// via [`return_data`].
    pub fn return_value(self) -> V {
        self.value
            .unwrap_or_else(|env_err| {
                panic!(
                    "Decoding dry run result to ink! message return type failed: {}",
                    env_err
                )
            })
            .unwrap_or_else(|lang_err| {
                panic!(
                    "Encountered a `LangError` while decoding dry run result to ink! message: {:?}",
                    lang_err
                )
            })
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
impl<C, E, V> core::fmt::Debug for CallResult<C, E, V>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("CallResult")
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
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
                f.write_str(&format!("ContractNotFound: {}", name))
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
            Error::Balance(msg) => write!(f, "Balance: {}", msg),
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
    contracts: BTreeMap<String, ContractMetadata>,
}

impl<C, E> Client<C, E>
where
    C: subxt::Config,
    C::AccountId: Into<C::Address> + serde::de::DeserializeOwned,
    C::Address: From<C::AccountId>,
    C::Signature: From<sr25519::Signature>,
    <C::Signature as Verify>::Signer: From<sr25519::Public>,
    <C::ExtrinsicParams as ExtrinsicParams<C::Index, C::Hash>>::OtherParams: Default,
    <C::Signature as Verify>::Signer:
        From<sr25519::Public> + IdentifyAccount<AccountId = C::AccountId>,
    sr25519::Signature: Into<C::Signature>,

    E: Environment,
    E::AccountId: Debug,
    E::Balance: Debug + scale::Encode + serde::Serialize,
    E::Hash: Debug + scale::Encode,

    Call<E, E::Balance>: scale::Encode,
    InstantiateWithCode<E::Balance>: scale::Encode,
{
    /// Creates a new [`Client`] instance.
    pub async fn new(url: &str, contracts: impl IntoIterator<Item = &str>) -> Self {
        let client = subxt::OnlineClient::from_url(url)
            .await
            .unwrap_or_else(|err| {
                if let subxt::Error::Rpc(subxt::error::RpcError::ClientError(_)) = err {
                    let error_msg = format!("Error establishing connection to a node at {}. Make sure you run a node behind the given url!", url);
                    log_error(&error_msg);
                    panic!("{}", error_msg);
                }
                log_error(
                    "Unable to create client! Please check that your node is running.",
                );
                panic!("Unable to create client: {:?}", err);
            });
        let contracts = contracts
            .into_iter()
            .map(|path| {
                let path = Path::new(path);
                let contract = ContractMetadata::load(path).unwrap_or_else(|err| {
                    panic!(
                        "Error loading contract metadata {}: {:?}",
                        path.display(),
                        err
                    )
                });
                (contract.contract.name.clone(), contract)
            })
            .collect();

        Self {
            api: ContractsApi::new(client, url).await,
            contracts,
        }
    }

    /// This function extracts the metadata of the contract at the file path
    /// `target/ink/$contract_name.contract`.
    ///
    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// Calling this function multiple times is idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    pub async fn instantiate<Args, R>(
        &mut self,
        contract_name: &str,
        signer: &Signer<C>,
        constructor: CreateBuilderPartial<E, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    where
        Args: scale::Encode,
    {
        let contract_metadata = self
            .contracts
            .get(contract_name)
            .ok_or_else(|| Error::ContractNotFound(contract_name.to_owned()))?;
        let code = crate::utils::extract_wasm(contract_metadata);
        let ret = self
            .exec_instantiate(signer, code, constructor, value, storage_deposit_limit)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }

    /// Dry run contract instantiation using the given constructor.
    pub async fn instantiate_dry_run<Args, R>(
        &mut self,
        contract_name: &str,
        signer: &Signer<C>,
        constructor: CreateBuilderPartial<E, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<C::AccountId, E::Balance>
    where
        Args: scale::Encode,
    {
        let contract_metadata = self
            .contracts
            .get(contract_name)
            .unwrap_or_else(|| panic!("Unknown contract {}", contract_name));
        let code = crate::utils::extract_wasm(contract_metadata);
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

    /// Executes an `instantiate_with_code` call and captures the resulting events.
    async fn exec_instantiate<Args, R>(
        &mut self,
        signer: &Signer<C>,
        code: Vec<u8>,
        constructor: CreateBuilderPartial<E, Args, R>,
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
                panic!("unable to unwrap event: {:?}", err);
            });

            if let Some(instantiated) = evt
                .as_event::<ContractInstantiatedEvent<E>>()
                .unwrap_or_else(|err| {
                    panic!("event conversion to `Instantiated` failed: {:?}", err);
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
                    "extrinsic for instantiate failed: {:?}",
                    dispatch_error
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
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|err| panic!("unable to get unix time: {}", err))
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
        let contract_metadata = self
            .contracts
            .get(contract_name)
            .ok_or_else(|| Error::ContractNotFound(contract_name.to_owned()))?;
        let code = crate::utils::extract_wasm(contract_metadata);
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
        log_info(&format!("upload dry run: {:?}", dry_run));
        if dry_run.is_err() {
            return Err(Error::UploadDryRun(dry_run))
        }

        let tx_events = self.api.upload(signer, code, storage_deposit_limit).await;

        let mut hash = None;
        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {:?}", err);
            });

            if let Some(uploaded) =
                evt.as_event::<CodeStoredEvent<E>>().unwrap_or_else(|err| {
                    panic!("event conversion to `Uploaded` failed: {:?}", err);
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
                log_error(&format!(
                    "extrinsic for upload failed: {:?}",
                    dispatch_error
                ));
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
                    .unwrap_or_else(|err| panic!("must have worked: {:?}", err))
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

        let dry_run = self
            .api
            .call_dry_run(signer.account_id().clone(), &message, value, None)
            .await;
        log_info(&format!("call dry run: {:?}", &dry_run.result));
        log_info(&format!(
            "call dry run debug message: {}",
            String::from_utf8_lossy(&dry_run.debug_message)
        ));
        if dry_run.result.is_err() {
            return Err(Error::CallDryRun(dry_run))
        }

        let tx_events = self
            .api
            .call(
                sp_runtime::MultiAddress::Id(message.account_id().clone()),
                value,
                dry_run.gas_required,
                storage_deposit_limit,
                message.exec_input().to_vec(),
                signer,
            )
            .await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {:?}", err);
            });

            if is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error = subxt::error::DispatchError::decode_from(
                    evt.field_bytes(),
                    &metadata,
                );
                log_error(&format!("extrinsic for call failed: {:?}", dispatch_error));
                return Err(Error::CallExtrinsic(dispatch_error))
            }
        }

        let bytes = &dry_run.result.as_ref().unwrap().data;
        let value: Result<MessageResult<RetType>, scale::Error> =
            scale::Decode::decode(&mut bytes.as_ref());

        Ok(CallResult {
            value,
            data: bytes.clone(),
            dry_run,
            events: tx_events,
        })
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
            .fetch_or_default(&account_addr, None)
            .await
            .unwrap_or_else(|err| {
                panic!("unable to fetch balance: {:?}", err);
            })
            .to_value()
            .unwrap_or_else(|err| {
                panic!("unable to decode account info: {:?}", err);
            });

        let account_data = get_composite_field_value(&account, "data")?;
        let balance = get_composite_field_value(account_data, "free")?;
        let balance = balance.as_u128().ok_or_else(|| {
            Error::Balance(format!("{:?} should convert to u128", balance))
        })?;
        let balance = E::Balance::try_from(balance).map_err(|_| {
            Error::Balance(format!("{:?} failed to convert from u128", balance))
        })?;

        log_info(&format!(
            "balance of contract {:?} is {:?}",
            account_id, balance
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
                Error::Balance(format!("No field named '{}' found", field_name))
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
