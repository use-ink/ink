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
        ContractDryCallResult,
        ContractDryInstantiateResult,
        InstantiateWithCode,
    },
    ContractsApi,
    InkConstructor,
    InkMessage,
    Signer,
};
use crate::Environment;

use sp_runtime::traits::{
    IdentifyAccount,
    Verify,
};
use subxt::{
    ext::bitvec::macros::internal::funty::Fundamental,
    metadata::DecodeStaticType,
    rpc::NumberOrHex,
    storage::address::{
        StorageHasher,
        StorageMapKey,
        Yes,
    },
    tx::{
        ExtrinsicParams,
        TxEvents,
    },
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
    pub dry_run: ContractDryInstantiateResult<C, E>,
    /// Events that happened with the contract instantiation.
    pub events: TxEvents<C>,
}

/// We implement a custom `Debug` here, as to avoid requiring the trait
/// bound `Debug` for `E`.
// TODO(#xxx) Improve the `Debug` implementation.
impl<C, E> core::fmt::Debug for InstantiationResult<C, E>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("CallResult")
            .field("account_id", &self.account_id)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract call.
pub struct CallResult<C: subxt::Config, E: Environment> {
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: ContractDryCallResult<E>,
    /// Events that happened with the contract instantiation.
    pub events: TxEvents<C>,
}

/// We implement a custom `Debug` here, as to avoid requiring the trait
/// bound `Debug` for `E`.
// TODO(#xxx) Improve the `Debug` implementation.
impl<C, E> core::fmt::Debug for CallResult<C, E>
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
    InstantiateDryRun(ContractDryInstantiateResult<C, E>),
    /// The `instantiate_with_code` extrinsic failed.
    InstantiateExtrinsic(subxt::error::DispatchError),
    /// The `call` dry run failed.
    CallDryRun(ContractDryCallResult<E>),
    /// The `call` extrinsic failed.
    CallExtrinsic(subxt::error::DispatchError),
}

// We implement a custom `Debug` here, as to avoid requiring the trait
// bound `Debug` for `C`.
impl<C, E> core::fmt::Debug for Error<C, E>
where
    C: subxt::Config,
    E: Environment,
    <E as Environment>::Balance: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self {
            Error::InstantiateDryRun(_) => f.write_str("InstantiateDryRun"),
            Error::InstantiateExtrinsic(_) => f.write_str("InstantiateExtrinsic"),
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

/// The `Client` takes care of communicating with the node.
///
/// This node's RPC interface will be used for instantiating the contract
/// and interacting with it .
pub struct Client<C, E>
where
    C: subxt::Config,
    E: Environment,
{
    node_log: String,
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
    E::Balance: core::fmt::Debug
        + scale::Encode
        + TryFrom<NumberOrHex>
        + TryFrom<sp_rpc::number::NumberOrHex>,
    NumberOrHex: From<<E as Environment>::Balance>,

    Call<C, E::Balance>: scale::Encode,
    InstantiateWithCode<E::Balance>: scale::Encode,
{
    /// Creates a new [`Client`] instance.
    pub async fn new(url: &str, node_log: &str) -> Self {
        let client = subxt::OnlineClient::from_url(url)
            .await
            .unwrap_or_else(|err| {
                log_error(
                    "Unable to create client! Please check that your node is running.",
                );
                panic!("Unable to create client: {:?}", err);
            });

        Self {
            api: ContractsApi::new(client, url).await,
            node_log: node_log.to_string(),
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
        name: &str,
        constructor: CO,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    where
        CO: InkConstructor,
    {
        let root = std::env::var("CARGO_MANIFEST_DIR")
            .expect("`CARGO_MANIFEST_DIR` is not set in env");
        let contract_path = format!("target/ink/{}.contract", name);
        let contract_path: std::path::PathBuf = [&root, &contract_path].iter().collect();
        let reader = std::fs::File::open(&contract_path).unwrap_or_else(|err| {
            panic!("metadata path cannot be opened: {:?}", err);
        });
        let contract: contract_metadata::ContractMetadata =
            serde_json::from_reader(reader).map_err(|err| {
                panic!("error reading metadata: {:?}", err);
            })?;
        let code = contract
            .source
            .wasm
            .expect("contract bundle is missing `source.wasm`");

        log_info(&format!(
            "{:?} has {} KiB",
            contract_path,
            code.0.len() / 1024
        ));

        let nonce = self
            .api
            .client
            .rpc()
            .system_account_next_index(signer.account_id())
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "error getting next index for {:?}: {:?}",
                    signer.account_id(),
                    err
                );
            });
        log_info(&format!("nonce: {:?}", nonce));
        signer.set_nonce(nonce);

        let ret = self
            .exec_instantiate(signer, value, storage_deposit_limit, code.0, &constructor)
            .await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));

        Ok(ret)
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
        log_info(&format!("instantiating with selector: {:?}", CO::SELECTOR));
        <CO as scale::Encode>::encode_to(constructor, &mut data);

        let salt = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unable to get unix time")
            .as_millis()
            .as_u128()
            .to_le_bytes()
            .to_vec();

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
        signer.increment_nonce();

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
            account_id: account_id.expect("cannot extract account_id from events"),
            events: tx_events,
        })
    }

    /// Executes a `call` for the contract at `account_id`.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    pub async fn call(
        &self,
        signer: &mut Signer<C>,
        account_id: C::AccountId,
        contract_call: EncodedMessage,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<CallResult<C, E>, Error<C, E>> {
        let dry_run = self
            .api
            .call_dry_run(account_id.clone(), value, None, contract_call.0.clone())
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
        signer.increment_nonce();

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
                return Err(Error::InstantiateExtrinsic(dispatch_error))
            }
        }

        Ok(CallResult {
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

    /// Returns true if the `substrate-contracts-node` log under
    /// `/tmp/contracts-node.log` contains `msg`.
    pub fn node_log_contains(&self, msg: &str) -> bool {
        let output = std::process::Command::new("grep")
            .arg("-q")
            .arg(msg)
            .arg(&self.node_log)
            .spawn()
            .map_err(|err| {
                format!("ERROR while executing `grep` with {:?}: {:?}", msg, err)
            })
            .expect("failed to execute process")
            .wait_with_output()
            .expect("failed to receive output");
        output.status.success()
    }
}
