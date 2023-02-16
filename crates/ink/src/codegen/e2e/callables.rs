use ink_env::{
    call::{
        utils::{ReturnType, Set, Unset},
        Call, CallBuilder, CreateBuilder, CreateParams, ExecutionInput, FromAccountId,
    },
    Environment,
};
use sp_core::sr25519;
use std::{fmt::Debug, path::Path};
use subxt::{
    blocks::ExtrinsicEvents,
    config::ExtrinsicParams,
    events::EventDetails,
    ext::scale_value::{Composite, Value, ValueDef},
    tx::PairSigner,
};

use contract_metadata::ContractMetadata;
use pallet_contracts_primitives::{
    CodeUploadResult, ContractExecResult, ContractInstantiateResult,
};
use scale::Encode;
use std::collections::BTreeMap;

use super::{
    builders::{constructor_exec_input, finalise_constructor, CreateBuilderPartial},
    log_error, log_info, log_prefix, utils, ContractsApi, Signer,
};

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

pub struct ConstructorCallable<'a, ContractRef, Args, R, E, C>
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
    constructor: CreateParams<E, ContractRef, Args, Vec<u8>, R>,
    value: E::Balance,
    storage_deposit_limit: Option<E::Balance>,
    contracts: &'a BTreeMap<String, ContractMetadata>,
    api: &'a ContractsApi<C, E>,
}

impl<'a, ContractRef, Args, R, E, C> ConstructorCallable<'a, ContractRef, Args, R, E, C>
where
    Args: Encode,
    E::Balance: Debug + scale::HasCompact + serde::Serialize,
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
    //TODO: need to figure out how to pass contracts and API in inside the test body
    pub fn new(
        constructor: CreateBuilderPartial<E, ContractRef, Args, R>,
        contracts: &'a BTreeMap<String, ContractMetadata>,
        api: &'a ContractsApi<C, E>,
    ) -> Self {
        let constructor = finalise_constructor(constructor);
        Self {
            contracts,
            api,
            constructor,
            value: 0u8.into(),
            storage_deposit_limit: None,
        }
    }

    pub fn with_value(&mut self, value: E::Balance) {
        self.value = value;
    }

    pub fn with_storage_deposit_limit(&mut self, storage_deposit_limit: E::Balance) {
        self.storage_deposit_limit = Some(storage_deposit_limit);
    }

    pub async fn call(
        mut self,
        contract_name: &str,
        signer: &Signer<C>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    where
        Args: scale::Encode,
    {
        let contract_metadata = self
            .contracts
            .get(contract_name)
            .ok_or_else(|| Error::ContractNotFound(contract_name.to_owned()))?;
        let code = super::utils::extract_wasm(contract_metadata);
        let ret = self.exec_instantiate(signer, code).await?;
        log_info(&format!("instantiated contract at {:?}", ret.account_id));
        Ok(ret)
    }

    pub async fn call_dry_run(
        self,
        contract_name: &str,
        signer: &PairSigner<C, sr25519::Pair>,
    ) -> ContractInstantiateResult<C::AccountId, E::Balance>
    where
        Args: scale::Encode,
    {
        let contract_metadata = self
            .contracts
            .get(contract_name)
            .unwrap_or_else(|| panic!("Unknown contract {contract_name}"));
        let code = super::utils::extract_wasm(contract_metadata);
        let data = constructor_exec_input(&self.constructor);

        let salt = Self::salt();
        self.api
            .instantiate_with_code_dry_run(
                self.value,
                self.storage_deposit_limit,
                code,
                data,
                salt,
                signer,
            )
            .await
    }

    /// Executes an `instantiate_with_code` call and captures the resulting events.
    async fn exec_instantiate(
        &mut self,
        signer: &Signer<C>,
        code: Vec<u8>,
    ) -> Result<InstantiationResult<C, E>, Error<C, E>>
    where
        Args: scale::Encode,
    {
        let salt = Self::salt();
        let data = constructor_exec_input(&self.constructor);

        // dry run the instantiate to calculate the gas limit
        let dry_run = self
            .api
            .instantiate_with_code_dry_run(
                self.value,
                self.storage_deposit_limit,
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
            return Err(Error::InstantiateDryRun(dry_run));
        }

        let tx_events = self
            .api
            .instantiate_with_code(
                self.value,
                dry_run.gas_required,
                self.storage_deposit_limit,
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
            } else if Self::is_extrinsic_failed_event(&evt) {
                let metadata = self.api.client.metadata();
                let dispatch_error = subxt::error::DispatchError::decode_from(
                    evt.field_bytes(),
                    &metadata,
                );
                log_error(&format!(
                    "extrinsic for instantiate failed: {dispatch_error:?}"
                ));
                return Err(Error::InstantiateExtrinsic(dispatch_error));
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

    /// Returns true if the give event is System::Extrinsic failed.
    fn is_extrinsic_failed_event(event: &EventDetails) -> bool {
        event.pallet_name() == "System" && event.variant_name() == "ExtrinsicFailed"
    }
}
