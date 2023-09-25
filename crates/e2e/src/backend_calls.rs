use ink_env::Environment;
use pallet_contracts_primitives::ContractInstantiateResult;
use scale::{
    Decode,
    Encode,
};

use crate::{
    builders::CreateBuilderPartial,
    CallBuilderFinal,
    CallDryRunResult,
    CallResult,
    ContractsBackend,
    InstantiationResult,
};

use super::Keypair;

pub struct CallBuilder<'a, E, Args, RetType, CB>
where
    E: Environment,
    Args: Sync + Encode,
    RetType: Send + Decode,

    CB: ContractsBackend<E>,
{
    client: &'a mut CB,
    caller: &'a Keypair,
    message: &'a CallBuilderFinal<E, Args, RetType>,
    value: E::Balance,
    extra_gas_portion: Option<usize>,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, Args, RetType, CB> CallBuilder<'a, E, Args, RetType, CB>
where
    E: Environment,
    Args: Sync + Encode,
    RetType: Send + Decode,
    E::Balance: Clone,

    CB: ContractsBackend<E>,
{
    /// Initialize a call builder with essential values.
    pub fn new(
        client: &'a mut CB,
        caller: &'a Keypair,
        message: &'a CallBuilderFinal<E, Args, RetType>,
    ) -> CallBuilder<'a, E, Args, RetType, CB>
    where
        E::Balance: From<u32>,
    {
        Self {
            client,
            caller,
            message,
            value: 0u32.into(),
            extra_gas_portion: None,
            storage_deposit_limit: None,
        }
    }

    /// Provide value with a call
    pub fn value(&mut self, value: E::Balance) {
        self.value = value;
    }

    /// Increases the gas limit marginally by a specified percent.
    /// Useful when the message's gas usage depends on the runtime state
    /// and the dry run does not produce an accurate gas estimate.
    ///
    /// # Example
    ///
    /// With dry run gas estimate of `100` units and `5`% extra gas portion specified,
    /// the set gas limit becomes `105` units
    pub fn extra_gas_portion(&mut self, per_cent: usize) {
        if per_cent == 0 {
            self.extra_gas_portion = None
        } else {
            self.extra_gas_portion = Some(per_cent)
        }
    }

    /// Specify the max amount of funds that can be charged for storage.
    pub fn storage_deposit_limit(&mut self, storage_deposit_limit: E::Balance) {
        if storage_deposit_limit == 0u32.into() {
            self.storage_deposit_limit = None
        } else {
            self.storage_deposit_limit = Some(storage_deposit_limit)
        }
    }

    /// Submit the call for the on-chain execution.
    pub async fn submit(
        &mut self,
    ) -> Result<CallResult<E, RetType, CB::EventLog>, CB::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        CB::call(
            self.client,
            self.caller,
            self.message,
            self.value,
            self.extra_gas_portion,
            self.storage_deposit_limit,
        )
        .await
    }

    /// Dry run the call.
    pub async fn submit_dry_run(&mut self) -> CallDryRunResult<E, RetType>
    where
        CallBuilderFinal<E, Args, RetType>: Clone,
    {
        CB::call_dry_run(
            self.client,
            self.caller,
            self.message,
            self.value,
            self.storage_deposit_limit,
        )
        .await
    }
}

pub struct InstantiateBuilder<'a, E, Contract, Args, R, CB>
where
    E: Environment,
    Args: Send + Encode,

    CB: ContractsBackend<E>,
{
    client: &'a mut CB,
    caller: &'a Keypair,
    contract_name: &'a str,
    constructor: CreateBuilderPartial<E, Contract, Args, R>,
    value: E::Balance,
    extra_gas_portion: Option<usize>,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, Contract, Args, R, CB> InstantiateBuilder<'a, E, Contract, Args, R, CB>
where
    E: Environment,
    Args: Send + Encode,

    CB: ContractsBackend<E>,
{
    /// Initialize a call builder with essential values.
    pub fn new(
        client: &'a mut CB,
        caller: &'a Keypair,
        contract_name: &'a str,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
    ) -> InstantiateBuilder<'a, E, Contract, Args, R, CB>
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
            storage_deposit_limit: None,
        }
    }

    /// Provide value with a call
    pub fn value(&mut self, value: E::Balance) {
        self.value = value;
    }

    /// Increases the gas limit marginally by a specified percent.
    /// Useful when the message's gas usage depends on the runtime state
    /// and the dry run does not produce an accurate gas estimate.
    ///
    /// # Example
    ///
    /// With dry run gas estimate of `100` units and `5`% extra gas portion specified,
    /// the set gas limit becomes `105` units
    pub fn extra_gas_portion(&mut self, per_cent: usize) {
        if per_cent == 0 {
            self.extra_gas_portion = None
        } else {
            self.extra_gas_portion = Some(per_cent)
        }
    }

    /// Specify the max amount of funds that can be charged for storage.
    pub fn storage_deposit_limit(&mut self, storage_deposit_limit: E::Balance) {
        if storage_deposit_limit == 0u32.into() {
            self.storage_deposit_limit = None
        } else {
            self.storage_deposit_limit = Some(storage_deposit_limit)
        }
    }

    /// Submit the instantiate call for the on-chain execution.
    pub async fn submit(self) -> Result<InstantiationResult<E, CB::EventLog>, CB::Error> {
        CB::instantiate(
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

    /// Dry run the instantiate call.
    pub async fn submit_dry_run(
        self,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()> {
        CB::instantiate_dry_run(
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
