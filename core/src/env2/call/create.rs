// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use core::marker::PhantomData;

use crate::{
    env2::{
        call::{
            CallData,
            Selector,
        },
        errors::CreateError,
        Env,
        EnvAccessMut,
        EnvTypes,
    },
    memory::vec::Vec,
};

pub mod state {
    pub use crate::env2::call::state::{
        Sealed,
        Unsealed,
    };

    /// Type state to indicate that the `code_hash` for cross-contract
    /// instantiation has already been provided.
    pub enum CodeHashAssigned {}
    /// Type state to indicate that the `code_hash` for cross-contract
    /// instantitation has not yet been provided.
    pub enum CodeHashUnassigned {}
}

/// Contracts that can be contructed from an `AccountId`
///
/// # Note
///
/// This is needed because of conflicting implementations of `From<T> for T`
/// in the generated code of `ink_lang`.
pub trait FromAccountId<E>
where
    E: EnvTypes,
{
    /// Creates the contract instance from the account ID of the already instantiated contract.
    fn from_account_id(account_id: <E as EnvTypes>::AccountId) -> Self;
}

/// Builds up contract instantiations.
pub struct CreateParams<E, C>
where
    E: EnvTypes,
{
    /// The code hash of the created contract.
    code_hash: E::Hash,
    /// The maximum gas costs allowed for the instantiation.
    gas_limit: u64,
    /// The transferred value for the newly created contract.
    value: E::Balance,
    /// The input data for the instantation.
    call_data: CallData,
    /// The type of the instantiated contract.
    contract_marker: PhantomData<fn() -> C>,
}

/// Builds up contract instantiations.
pub struct CreateBuilder<E, C, Seal, CodeHash>
where
    E: EnvTypes,
{
    /// The parameters of the cross-contract instantiation.
    params: CreateParams<E, C>,
    /// Seal state.
    state: PhantomData<fn() -> (Seal, CodeHash)>,
}

impl<E, C> CreateParams<E, C>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    pub fn code_hash(&self) -> &E::Hash {
        &self.code_hash
    }

    /// The gas limit for the contract instantiation.
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
    /// The endowment for the instantiated contract.
    pub fn endowment(&self) -> &E::Balance {
        &self.value
    }

    /// The raw encoded input data.
    pub fn input_data(&self) -> &CallData {
        &self.call_data
    }
}

impl<E, C> CreateParams<E, C>
where
    E: EnvTypes,
    E::Hash: Default,
    E::Balance: Default,
{
    /// Creates the default set of initial create parameters.
    fn new(selector: Selector) -> Self {
        Self {
            code_hash: Default::default(),
            gas_limit: 0,
            value: Default::default(),
            call_data: CallData::new(selector),
            contract_marker: Default::default(),
        }
    }

    /// Creates a new create builder without setting any presets.
    pub fn build(
        selector: Selector,
    ) -> CreateBuilder<E, C, state::Unsealed, state::CodeHashUnassigned> {
        CreateBuilder {
            params: CreateParams::new(selector),
            state: Default::default(),
        }
    }
}

impl<E, C, Seal, CodeHash> CreateBuilder<E, C, Seal, CodeHash>
where
    E: EnvTypes,
{
    /// Sets the maximumly allowed gas costs for the call.
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.params.gas_limit = gas_limit;
        self
    }

    /// Sets the value transferred upon the execution of the call.
    pub fn value(mut self, value: E::Balance) -> Self {
        self.params.value = value;
        self
    }
}

impl<E, C, Seal> CreateBuilder<E, C, Seal, state::CodeHashUnassigned>
where
    E: EnvTypes,
{
    /// Using the given code hash.
    pub fn using_code(
        mut self,
        code_hash: E::Hash,
    ) -> CreateBuilder<E, C, Seal, state::CodeHashAssigned> {
        self.params.code_hash = code_hash;
        CreateBuilder {
            params: self.params,
            state: Default::default(),
        }
    }
}

impl<E, C, CodeHash> CreateBuilder<E, C, state::Unsealed, CodeHash>
where
    E: EnvTypes,
{
    /// Pushes an argument to the inputs of the call.
    pub fn push_arg<A>(mut self, arg: &A) -> Self
    where
        A: scale::Encode,
    {
        self.params.call_data.push_arg(arg);
        self
    }

    /// Seals the create builder to prevent further arguments.
    pub fn seal(self) -> CreateBuilder<E, C, state::Sealed, CodeHash> {
        CreateBuilder {
            params: self.params,
            state: Default::default(),
        }
    }
}

impl<E, C, Seal> CreateBuilder<E, C, Seal, state::CodeHashAssigned>
where
    E: Env,
    C: FromAccountId<E>,
{
    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Note
    ///
    /// Prefer using [`create_using`] whenever possible because it is more efficient.
    pub fn create(self) -> Result<C, CreateError> {
        E::create_contract(&mut Vec::new(), &self.params)
            .map(FromAccountId::from_account_id)
            .map_err(|_| CreateError)
    }

    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Note
    ///
    /// Prefer using this over [`create`] whenever possible because it is more efficient.
    pub fn create_using(
        self,
        env_access: &mut EnvAccessMut<E>,
    ) -> Result<C, CreateError> {
        env_access
            .create_contract(&self.params)
            .map(FromAccountId::from_account_id)
            .map_err(|_| CreateError)
    }
}
