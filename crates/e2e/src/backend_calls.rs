use ink_env::Environment;
use pallet_contracts_primitives::ContractInstantiateResult;
use scale::{
    Decode,
    Encode,
};
use sp_weights::Weight;

use crate::{
    backend::BuilderClient,
    builders::CreateBuilderPartial,
    CallBuilderFinal,
    CallDryRunResult,
    CallResult,
    ContractsBackend,
    InstantiationResult,
    UploadResult,
};

use super::Keypair;

/// Allows to build an end-to-end call using a builder pattern.
pub struct CallBuilder<'a, E, Args, RetType, B>
where
    E: Environment,
    Args: Encode + Clone,
    RetType: Send + Decode,

    B: BuilderClient<E>,
{
    client: &'a mut B,
    caller: &'a Keypair,
    message: &'a CallBuilderFinal<E, Args, RetType>,
    value: E::Balance,
    extra_gas_portion: Option<u64>,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, Args, RetType, B> CallBuilder<'a, E, Args, RetType, B>
where
    E: Environment,
    Args: Sync + Encode + Clone,
    RetType: Send + Decode,

    B: BuilderClient<E>,
{
    /// Initialize a call builder with defaults values.
    pub fn new(
        client: &'a mut B,
        caller: &'a Keypair,
        message: &'a CallBuilderFinal<E, Args, RetType>,
    ) -> CallBuilder<'a, E, Args, RetType, B>
    where
        E::Balance: From<u32>,
    {
        Self {
            client,
            caller,
            message,
            value: 0u32.into(),
            extra_gas_portion: None,
            gas_limit: None,
            storage_deposit_limit: None,
        }
    }

    /// Provide value with a call
    pub fn value(&mut self, value: E::Balance) -> &mut Self {
        self.value = value;
        self
    }

    /// Increases the gas limit marginally by a specified percent.
    /// Useful when the message's gas usage depends on the runtime state
    /// and the dry run does not produce an accurate gas estimate.
    ///
    /// # Example
    ///
    /// With dry run gas estimate of `100` units and `5`% extra gas portion specified,
    /// the set gas limit becomes `105` units
    pub fn extra_gas_portion(&mut self, per_cent: u64) -> &mut Self {
        if per_cent == 0 {
            self.extra_gas_portion = None
        } else {
            self.extra_gas_portion = Some(per_cent)
        }
        self
    }

    /// Specifies the raw gas limit as part of the call.
    ///
    /// # Notes
    ///
    /// Overwrites any values specified for `extra_gas_portion`.
    ///  The gas estimate fro dry-run will be ignored.
    pub fn gas_limit(&mut self, limit: Weight) -> &mut Self {
        if limit == Weight::from(0) {
            self.gas_limit = None
        } else {
            self.gas_limit = Some(limit)
        }
        self
    }

    /// Specify the max amount of funds that can be charged for storage.
    pub fn storage_deposit_limit(
        &mut self,
        storage_deposit_limit: E::Balance,
    ) -> &mut Self {
        if storage_deposit_limit == 0u32.into() {
            self.storage_deposit_limit = None
        } else {
            self.storage_deposit_limit = Some(storage_deposit_limit)
        }
        self
    }

    /// Submit the call for the on-chain execution.
    ///
    /// This will automatically run a dry-run call, and use `extra_gas_portion`
    /// to add a margin to the gas limit.
    pub async fn submit(
        &mut self,
    ) -> Result<CallResult<E, RetType, B::EventLog>, B::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        if let Some(limit) = self.gas_limit {
            B::call_with_gas_limit(
                self.client,
                self.caller,
                self.message,
                self.value,
                limit,
                self.storage_deposit_limit,
            )
            .await
        } else {
            B::call_with_gas_margin(
                self.client,
                self.caller,
                self.message,
                self.value,
                self.extra_gas_portion,
                self.storage_deposit_limit,
            )
            .await
        }
    }

    /// Dry run the call.
    pub async fn dry_run(&mut self) -> CallDryRunResult<E, RetType>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        B::bare_call_dry_run(
            self.client,
            self.caller,
            self.message,
            self.value,
            self.storage_deposit_limit,
        )
        .await
    }
}

/// Allows to build an end-to-end instantiation call using a builder pattern.
pub struct InstantiateBuilder<'a, E, Contract, Args, R, B>
where
    E: Environment,
    Args: Encode + Clone,
    Contract: Clone,

    B: ContractsBackend<E>,
{
    client: &'a mut B,
    caller: &'a Keypair,
    contract_name: &'a str,
    constructor: &'a mut CreateBuilderPartial<E, Contract, Args, R>,
    value: E::Balance,
    extra_gas_portion: Option<u64>,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, Contract, Args, R, B> InstantiateBuilder<'a, E, Contract, Args, R, B>
where
    E: Environment,
    Args: Encode + Clone + Send + Sync,
    Contract: Clone,

    B: BuilderClient<E>,
{
    /// Initialize a call builder with essential values.
    pub fn new(
        client: &'a mut B,
        caller: &'a Keypair,
        contract_name: &'a str,
        constructor: &'a mut CreateBuilderPartial<E, Contract, Args, R>,
    ) -> InstantiateBuilder<'a, E, Contract, Args, R, B>
    where
        E::Balance: From<u32>,
    {
        Self {
            client,
            caller,
            contract_name,
            constructor,
            value: 0u32.into(),
            extra_gas_portion: None,
            gas_limit: None,
            storage_deposit_limit: None,
        }
    }

    /// Provide value with a call
    pub fn value(&mut self, value: E::Balance) -> &mut Self {
        self.value = value;
        self
    }

    /// Increases the gas limit marginally by a specified percent.
    /// Useful when the message's gas usage depends on the runtime state
    /// and the dry run does not produce an accurate gas estimate.
    ///
    /// # Example
    ///
    /// With dry run gas estimate of `100` units and `5`% extra gas portion specified,
    /// the set gas limit becomes `105` units
    pub fn extra_gas_portion(&mut self, per_cent: u64) -> &mut Self {
        if per_cent == 0 {
            self.extra_gas_portion = None
        } else {
            self.extra_gas_portion = Some(per_cent)
        }
        self
    }

    /// Specifies the raw gas limit as part of the call.
    ///
    /// # Notes
    ///
    /// Overwrites any values specified for `extra_gas_portion`.
    /// The gas estimate fro dry-run will be ignored.
    pub fn gas_limit(&mut self, limit: Weight) -> &mut Self {
        if limit == Weight::from(0) {
            self.gas_limit = None
        } else {
            self.gas_limit = Some(limit)
        }
        self
    }

    /// Specify the max amount of funds that can be charged for storage.
    pub fn storage_deposit_limit(
        &mut self,
        storage_deposit_limit: E::Balance,
    ) -> &mut Self {
        if storage_deposit_limit == 0u32.into() {
            self.storage_deposit_limit = None
        } else {
            self.storage_deposit_limit = Some(storage_deposit_limit)
        }
        self
    }

    /// Submit the instantiate call for the on-chain execution.
    ///
    /// This will automatically run a dry-run call, and use `extra_gas_portion`
    /// to add a margin to the gas limit.
    pub async fn submit(
        &mut self,
    ) -> Result<InstantiationResult<E, B::EventLog>, B::Error> {
        if let Some(limit) = self.gas_limit {
            B::instantiate_with_gas_limit(
                self.client,
                self.contract_name,
                self.caller,
                self.constructor,
                self.value,
                limit,
                self.storage_deposit_limit,
            )
            .await
        } else {
            B::instantiate_with_gas_margin(
                self.client,
                self.contract_name,
                self.caller,
                self.constructor,
                self.value,
                self.extra_gas_portion,
                self.storage_deposit_limit,
            )
            .await
        }
    }

    /// Dry run the instantiate call.
    pub async fn dry_run(
        &mut self,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()> {
        B::bare_instantiate_dry_run(
            self.client,
            self.contract_name,
            self.caller,
            self.constructor,
            self.value,
            self.storage_deposit_limit,
        )
        .await
    }
}

/// Allows to build an end-to-end upload call using a builder pattern.
pub struct UploadBuilder<'a, E, B>
where
    E: Environment,
    B: BuilderClient<E>,
{
    client: &'a mut B,
    contract_name: &'a str,
    caller: &'a Keypair,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, B> UploadBuilder<'a, E, B>
where
    E: Environment,
    B: BuilderClient<E>,
{
    /// Initialize an upload builder with essential values.
    pub fn new(client: &'a mut B, contract_name: &'a str, caller: &'a Keypair) -> Self {
        Self {
            client,
            contract_name,
            caller,
            storage_deposit_limit: None,
        }
    }

    /// Specify the max amount of funds that can be charged for storage.
    pub fn storage_deposit_limit(
        &mut self,
        storage_deposit_limit: E::Balance,
    ) -> &mut Self {
        if storage_deposit_limit == 0u32.into() {
            self.storage_deposit_limit = None
        } else {
            self.storage_deposit_limit = Some(storage_deposit_limit)
        }
        self
    }

    /// Execute the upload.
    pub async fn submit(&mut self) -> Result<UploadResult<E, B::EventLog>, B::Error> {
        B::bare_upload(
            self.client,
            self.contract_name,
            self.caller,
            self.storage_deposit_limit,
        )
        .await
    }
}
