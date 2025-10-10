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

use std::marker::PhantomData;

use ink_env::{
    Environment,
    call::utils::DecodeMessageResult,
};
use ink_primitives::abi::AbiEncodeWith;
use sp_weights::Weight;

use super::{
    InstantiateDryRunResult,
    Keypair,
};
use crate::{
    CallBuilderFinal,
    CallDryRunResult,
    CallResult,
    ContractsBackend,
    H256,
    InstantiationResult,
    UploadResult,
    backend::BuilderClient,
    builders::CreateBuilderPartial,
};

/// Allows to build an end-to-end call using a builder pattern.
pub struct CallBuilder<'a, E, Args, RetType, B, Abi>
where
    E: Environment,
    Args: AbiEncodeWith<Abi> + Clone,
    RetType: Send + DecodeMessageResult<Abi>,
    B: BuilderClient<E>,
    Abi: Clone,
{
    client: &'a mut B,
    caller: &'a Keypair,
    message: &'a CallBuilderFinal<E, Args, RetType, Abi>,
    value: E::Balance,
    extra_gas_portion: Option<u64>,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, Args, RetType, B, Abi> CallBuilder<'a, E, Args, RetType, B, Abi>
where
    E: Environment,
    Args: Sync + AbiEncodeWith<Abi> + Clone,
    RetType: Send + DecodeMessageResult<Abi>,
    B: BuilderClient<E>,
    Abi: Sync + Clone,
{
    /// Initialize a call builder with defaults values.
    pub fn new(
        client: &'a mut B,
        caller: &'a Keypair,
        message: &'a CallBuilderFinal<E, Args, RetType, Abi>,
    ) -> CallBuilder<'a, E, Args, RetType, B, Abi>
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
    /// The gas estimate from the dry-run will be ignored.
    pub fn gas_limit(&mut self, limit: Weight) -> &mut Self {
        if limit == Weight::from_parts(0, 0) {
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
        self.storage_deposit_limit = Some(storage_deposit_limit);
        self
    }

    /// Submit the call for the on-chain execution.
    ///
    /// This will automatically run a dry-run call, and use `extra_gas_portion`
    /// to add a margin to the gas limit.
    pub async fn submit(
        &mut self,
    ) -> Result<CallResult<E, RetType, B::EventLog, Abi>, B::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone,
    {
        let _map = B::map_account(self.client, self.caller).await; // todo will fail if instantiation happened before

        let dry_run = B::bare_call_dry_run(
            self.client,
            self.caller,
            self.message,
            self.value,
            self.storage_deposit_limit,
        )
        .await?;

        let gas_limit = if let Some(limit) = self.gas_limit {
            limit
        } else {
            let gas_required = dry_run.exec_result.gas_required;
            let proof_size = gas_required.proof_size();
            let ref_time = gas_required.ref_time();
            calculate_weight(proof_size, ref_time, self.extra_gas_portion)
        };

        let (events, trace) = B::bare_call(
            self.client,
            self.caller,
            self.message,
            self.value,
            gas_limit,
            dry_run.exec_result.storage_deposit.charge_or_zero(),
        )
        .await?;

        Ok(CallResult {
            dry_run,
            events,
            trace,
        })
    }

    /// Dry run the call.
    pub async fn dry_run(&mut self) -> Result<CallDryRunResult<E, RetType, Abi>, B::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone,
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
pub struct InstantiateBuilder<'a, E, Contract, Args, R, B, Abi>
where
    E: Environment,
    Args: AbiEncodeWith<Abi> + Clone,
    Contract: Clone,
    B: ContractsBackend<E>,
{
    client: &'a mut B,
    caller: &'a Keypair,
    contract_name: &'a str,
    constructor: &'a mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
    value: E::Balance,
    extra_gas_portion: Option<u64>,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<E::Balance>,
}

impl<'a, E, Contract, Args, R, B, Abi>
    InstantiateBuilder<'a, E, Contract, Args, R, B, Abi>
where
    E: Environment,
    Args: AbiEncodeWith<Abi> + Clone + Send + Sync,
    Contract: Clone,
    B: BuilderClient<E>,
    Abi: Send + Sync + Clone,
{
    /// Initialize a call builder with essential values.
    pub fn new(
        client: &'a mut B,
        caller: &'a Keypair,
        contract_name: &'a str,
        constructor: &'a mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
    ) -> InstantiateBuilder<'a, E, Contract, Args, R, B, Abi>
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
    /// The gas estimate from dry-run will be ignored.
    pub fn gas_limit(&mut self, limit: Weight) -> &mut Self {
        if limit == Weight::from_parts(0, 0) {
            self.gas_limit = None
        } else {
            self.gas_limit = Some(limit)
        }
        self
    }

    /// Specify the max amount of funds that can be charged for storage.
    ///
    /// *Important*: `None` means charging the maximum!
    pub fn storage_deposit_limit(
        &mut self,
        storage_deposit_limit: Option<E::Balance>,
    ) -> &mut Self {
        self.storage_deposit_limit = storage_deposit_limit;
        self
    }

    /// Submit the instantiate call for the on-chain execution.
    ///
    /// This will automatically run a dry-run call, and use `extra_gas_portion`
    /// to add a margin to the gas limit.
    pub async fn submit(
        &mut self,
    ) -> Result<InstantiationResult<E, B::EventLog, Abi>, B::Error> {
        // we have to make sure the account was mapped
        let _map = B::map_account(self.client, self.caller).await; // todo will fail if instantiation happened before

        let dry_run = B::bare_instantiate_dry_run(
            self.client,
            self.contract_name,
            self.caller,
            self.constructor,
            self.value,
            self.storage_deposit_limit.clone(),
        )
        .await?;

        let gas_limit = if let Some(limit) = self.gas_limit {
            limit
        } else {
            let gas_required = dry_run.contract_result.gas_required;
            let proof_size = gas_required.proof_size();
            let ref_time = gas_required.ref_time();
            calculate_weight(proof_size, ref_time, self.extra_gas_portion)
        };

        let instantiate_result = B::bare_instantiate(
            self.client,
            B::load_code(self.client, self.contract_name),
            self.caller,
            self.constructor,
            self.value,
            gas_limit,
            dry_run.contract_result.storage_deposit.charge_or_zero(),
        )
        .await?;

        Ok(InstantiationResult {
            addr: instantiate_result.addr,
            account_id: instantiate_result.account_id,
            dry_run,
            events: instantiate_result.events,
            trace: instantiate_result.trace,
            code_hash: instantiate_result.code_hash,
        })
    }

    /// Dry run the instantiate call.
    pub async fn dry_run(&mut self) -> Result<InstantiateDryRunResult<E, Abi>, B::Error> {
        B::bare_instantiate_dry_run(
            self.client,
            self.contract_name,
            self.caller,
            self.constructor,
            self.value,
            self.storage_deposit_limit.clone(),
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
        storage_deposit_limit: Option<E::Balance>,
    ) -> &mut Self {
        self.storage_deposit_limit = storage_deposit_limit;
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

/// Allows to build an end-to-end remove code call using a builder pattern.
pub struct RemoveCodeBuilder<'a, E, B>
where
    E: Environment,
    B: BuilderClient<E>,
{
    client: &'a mut B,
    caller: &'a Keypair,
    code_hash: crate::H256,
    _phantom: PhantomData<fn() -> E>,
}

impl<'a, E, B> RemoveCodeBuilder<'a, E, B>
where
    E: Environment,
    B: BuilderClient<E>,
{
    /// Initialize a remove code builder with essential values.
    pub fn new(client: &'a mut B, caller: &'a Keypair, code_hash: H256) -> Self {
        Self {
            client,
            caller,
            code_hash,
            _phantom: Default::default(),
        }
    }

    /// Submit the remove code extrinsic.
    pub async fn submit(&mut self) -> Result<B::EventLog, B::Error> {
        B::bare_remove_code(self.client, self.caller, self.code_hash).await
    }
}

fn calculate_weight(
    mut proof_size: u64,
    mut ref_time: u64,
    portion: Option<u64>,
) -> Weight {
    if let Some(m) = portion {
        ref_time += ref_time / 100 * m;
        proof_size += proof_size / 100 * m;
    }
    Weight::from_parts(ref_time, proof_size)
}
