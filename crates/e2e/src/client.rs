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
    client::api::runtime_types::{
        frame_system::AccountInfo,
        pallet_balances::AccountData,
    },
    log_error,
    log_info,
    sr25519,
    xts::{
        self,
        api,
        Call,
        InstantiateWithCode,
    },
    CodeUploadResult,
    ContractExecResult,
    ContractInstantiateResult,
    ContractsApi,
    InkConstructor,
    InkMessage,
    Signer,
};
use ink_env::Environment;

use sp_runtime::traits::{
    IdentifyAccount,
    Verify,
};
use subxt::{
    blocks::ExtrinsicEvents,
    ext::bitvec::macros::internal::funty::Fundamental,
    metadata::DecodeStaticType,
    storage::address::{
        StorageHasher,
        StorageMapKey,
        Yes,
    },
    tx::ExtrinsicParams,
};

/// An encoded `#[ink(message)]`.
#[derive(Clone)]
pub struct EncodedMessage(Vec<u8>);

impl EncodedMessage {
    fn new<M: InkMessage>(call: &M) -> Self {
        let mut call_data = M::SELECTOR.to_vec();
        <M as scale::Encode>::encode_to(call, &mut call_data);
        Self(call_data)
    }
}

impl<M> From<M> for EncodedMessage
where
    M: InkMessage,
{
    fn from(msg: M) -> Self {
        EncodedMessage::new(&msg)
    }
}

/// Result of a contract instantiation.
pub struct InstantiationResult<C: subxt::Config, E: Environment> {
    /// The account id at which the contract was instantiated.
    pub account_id: C::AccountId,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: ContractInstantiateResult<C::AccountId, E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: ExtrinsicEvents<C>,
}

/// Result of a contract upload.
pub struct UploadResult<C: subxt::Config, E: Environment> {
    /// The hash with which the contract can be instantiated.
    pub code_hash: C::Hash,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: CodeUploadResult<C::Hash, E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: ExtrinsicEvents<C>,
}

/// We implement a custom `Debug` here, to avoid requiring the trait
/// bound `Debug` for `E`.
impl<C, E> core::fmt::Debug for UploadResult<C, E>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: core::fmt::Debug,
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
    <E as Environment>::Balance: core::fmt::Debug,
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
    /// Contains the return value of the called function.
    ///
    /// This field contains the decoded `data` from the dry-run,
    /// the raw data is available under `dry_run.result.data`.
    pub value: V,
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
    /// The `instantiate_with_code` dry run failed.
    InstantiateDryRun(ContractInstantiateResult<C::AccountId, E::Balance>),
    /// The `instantiate_with_code` extrinsic failed.
    InstantiateExtrinsic(subxt::error::DispatchError),
    /// The `upload` dry run failed.
    UploadDryRun(CodeUploadResult<C::Hash, E::Balance>),
    /// The `upload` extrinsic failed.
    UploadExtrinsic(subxt::error::DispatchError),
    /// The `call` dry run failed.
    CallDryRun(ContractExecResult<E::Balance>),
    /// The `call` extrinsic failed.
    CallExtrinsic(subxt::error::DispatchError),
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
        }
    }
}

/// A contract was successfully instantiated.
#[derive(Debug, scale::Decode, scale::Encode)]
struct ContractInstantiatedEvent<C: subxt::Config> {
    /// Account id of the deployer.
    pub deployer: C::AccountId,
    /// Account id where the contract was instantiated to.
    pub contract: C::AccountId,
}

impl<C> subxt::events::StaticEvent for ContractInstantiatedEvent<C>
where
    C: subxt::Config,
{
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "Instantiated";
}

/// Code with the specified hash has been stored.
#[derive(Debug, scale::Decode, scale::Encode)]
struct CodeStoredEvent<C: subxt::Config> {
    /// Hash under which the contract code was stored.
    pub code_hash: C::Hash,
}

impl<C> subxt::events::StaticEvent for CodeStoredEvent<C>
where
    C: subxt::Config,
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
    E::Balance: core::fmt::Debug + scale::Encode + serde::Serialize,

    Call<C, E::Balance>: scale::Encode,
    InstantiateWithCode<E::Balance>: scale::Encode,
{
    /// Creates a new [`Client`] instance.
    pub async fn new(url: &str) -> Self {
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

        Self {
            api: ContractsApi::new(client, url).await,
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
    pub async fn instantiate<CO>(
        &mut self,
        signer: &mut Signer<C>,
        constructor: CO,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    where
        CO: InkConstructor,
    {
        let code = crate::utils::extract_wasm(CO::CONTRACT_PATH);
        let ret = self
            .exec_instantiate(signer, value, storage_deposit_limit, code, &constructor)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }

    /// Dry run contract instantiation using the given constructor.
    pub async fn instantiate_dry_run<CO: InkConstructor>(
        &mut self,
        signer: &Signer<C>,
        constructor: &CO,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<C::AccountId, E::Balance>
    where
        CO: InkConstructor,
    {
        let mut data = CO::SELECTOR.to_vec();
        <CO as scale::Encode>::encode_to(constructor, &mut data);

        let code = crate::utils::extract_wasm(CO::CONTRACT_PATH);
        let salt = Self::salt();
        self.api
            .instantiate_with_code_dry_run(
                value,
                storage_deposit_limit,
                code.clone(),
                data.clone(),
                salt.clone(),
                signer,
            )
            .await
    }

    /// Executes an `instantiate_with_code` call and captures the resulting events.
    async fn exec_instantiate<CO: InkConstructor>(
        &mut self,
        signer: &mut Signer<C>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
        code: Vec<u8>,
        constructor: &CO,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>> {
        let mut data = CO::SELECTOR.to_vec();
        log_info(&format!(
            "instantiating with selector: {:02X?}",
            CO::SELECTOR
        ));
        <CO as scale::Encode>::encode_to(constructor, &mut data);

        let salt = Self::salt();

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
                .as_event::<ContractInstantiatedEvent<C>>()
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
            } else if evt
                .as_event::<xts::api::system::events::ExtrinsicFailed>()
                .unwrap_or_else(|err| {
                    panic!("event conversion to `ExtrinsicFailed` failed: {:?}", err)
                })
                .is_some()
            {
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

    /// This function extracts the metadata of the contract at the file path
    /// `target/ink/$contract_name.contract`.
    ///
    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// Calling this function multiple times is idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    pub async fn upload(
        &mut self,
        signer: &mut Signer<C>,
        contract_path: &str,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<C, E>, Error<C, E>> {
        let code = crate::utils::extract_wasm(contract_path);
        let ret = self
            .exec_upload(signer, code, storage_deposit_limit)
            .await?;
        log_info(&format!("contract stored with hash {:?}", ret.code_hash));
        Ok(ret)
    }

    /// Executes an `instantiate_with_code` call and captures the resulting events.
    async fn exec_upload(
        &mut self,
        signer: &mut Signer<C>,
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
                evt.as_event::<CodeStoredEvent<C>>().unwrap_or_else(|err| {
                    panic!("event conversion to `Uploaded` failed: {:?}", err);
                })
            {
                log_info(&format!(
                    "contract was uploaded with hash {:?}",
                    uploaded.code_hash
                ));
                hash = Some(uploaded.code_hash);
                break
            } else if evt
                .as_event::<xts::api::system::events::ExtrinsicFailed>()
                .unwrap_or_else(|err| {
                    panic!("event conversion to `ExtrinsicFailed` failed: {:?}", err)
                })
                .is_some()
            {
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
    pub async fn call<M>(
        &mut self,
        signer: &mut Signer<C>,
        account_id: C::AccountId,
        contract_call: M,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<C, E, <M as InkMessage>::ReturnType>, Error<C, E>>
    where
        M: InkMessage,
        <M as InkMessage>::ReturnType: scale::Decode,
    {
        let contract_call: EncodedMessage = contract_call.into();
        log_info(&format!("call: {:02X?}", contract_call.0));

        let dry_run = self
            .api
            .call_dry_run(
                signer.account_id().clone(),
                account_id.clone(),
                value,
                None,
                contract_call.0.clone(),
            )
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
                sp_runtime::MultiAddress::Id(account_id),
                value,
                dry_run.gas_required,
                storage_deposit_limit,
                contract_call.0.clone(),
                signer,
            )
            .await;

        for evt in tx_events.iter() {
            let evt = evt.unwrap_or_else(|err| {
                panic!("unable to unwrap event: {:?}", err);
            });

            if evt
                .as_event::<xts::api::system::events::ExtrinsicFailed>()
                .unwrap_or_else(|err| {
                    panic!("event conversion to `ExtrinsicFailed` failed: {:?}", err)
                })
                .is_some()
            {
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
        let value: <M as InkMessage>::ReturnType =
            scale::Decode::decode(&mut bytes.as_ref()).unwrap_or_else(|err| {
                panic!(
                    "decoding dry run result to ink! message return type failed: {}",
                    err
                )
            });

        Ok(CallResult {
            value,
            dry_run,
            events: tx_events,
        })
    }

    /// Returns the balance of `account_id`.
    pub async fn balance(
        &self,
        account_id: C::AccountId,
    ) -> Result<E::Balance, Error<C, E>> {
        let account_addr = subxt::storage::StaticStorageAddress::<
            DecodeStaticType<AccountInfo<C::Index, AccountData<E::Balance>>>,
            Yes,
            Yes,
            (),
        >::new(
            "System",
            "Account",
            vec![StorageMapKey::new(
                account_id.clone(),
                StorageHasher::Blake2_128Concat,
            )],
            Default::default(),
        )
        .unvalidated();

        let alice_pre: AccountInfo<C::Index, AccountData<E::Balance>> = self
            .api
            .client
            .storage()
            .fetch_or_default(&account_addr, None)
            .await
            .unwrap_or_else(|err| {
                panic!("unable to fetch balance: {:?}", err);
            });
        log_info(&format!(
            "balance of contract {:?} is {:?}",
            account_id, alice_pre
        ));
        Ok(alice_pre.data.free)
    }
}
