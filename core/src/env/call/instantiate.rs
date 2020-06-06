// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use crate::env::{
    call::{
        ArgsList,
        Argument,
        ArgumentList,
        EmptyArgumentList,
        ExecutionInput,
        Selector,
    },
    EnvTypes,
    Result,
};

pub mod state {
    pub use crate::env::call::state::{
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
pub trait FromAccountId<T>
where
    T: EnvTypes,
{
    /// Creates the contract instance from the account ID of the already instantiated contract.
    fn from_account_id(account_id: <T as EnvTypes>::AccountId) -> Self;
}

/// Builds up contract instantiations.
pub struct InstantiateParams<T, Args, C>
where
    T: EnvTypes,
{
    /// The code hash of the created contract.
    code_hash: T::Hash,
    /// The maximum gas costs allowed for the instantiation.
    gas_limit: u64,
    /// The endowment for the instantiated contract.
    endowment: T::Balance,
    /// The input data for the instantation.
    call_data: ExecutionInput<Args>,
    /// The type of the instantiated contract.
    contract_marker: PhantomData<fn() -> C>,
}

/// Builds up contract instantiations.
pub struct InstantiateBuilder<T, Args, C, Seal, CodeHash>
where
    T: EnvTypes,
{
    /// The parameters of the cross-contract instantiation.
    params: InstantiateParams<T, Args, C>,
    /// Seal state.
    state: PhantomData<fn() -> (Seal, CodeHash)>,
}

impl<T, Args, C> InstantiateParams<T, Args, C>
where
    T: EnvTypes,
{
    /// The code hash of the contract.
    #[inline]
    pub fn code_hash(&self) -> &T::Hash {
        &self.code_hash
    }

    /// The gas limit for the contract instantiation.
    #[inline]
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
    /// The endowment for the instantiated contract.
    #[inline]
    pub fn endowment(&self) -> &T::Balance {
        &self.endowment
    }

    /// The raw encoded input data.
    #[inline]
    pub fn input_data(&self) -> &ExecutionInput<Args> {
        &self.call_data
    }
}

impl<T, C> InstantiateParams<T, EmptyArgumentList, C>
where
    T: EnvTypes,
    T::Hash: Default,
    T::Balance: Default,
{
    /// Creates the default set of initial create parameters.
    fn new(selector: Selector) -> Self {
        Self {
            code_hash: Default::default(),
            gas_limit: 0,
            endowment: Default::default(),
            call_data: ExecutionInput::new(selector),
            contract_marker: Default::default(),
        }
    }

    /// Creates a new create builder without setting any presets.
    #[inline]
    pub fn build(
        selector: Selector,
    ) -> InstantiateBuilder<
        T,
        EmptyArgumentList,
        C,
        state::Unsealed,
        state::CodeHashUnassigned,
    > {
        InstantiateBuilder {
            params: InstantiateParams::new(selector),
            state: Default::default(),
        }
    }
}

impl<T, Args, C, Seal, CodeHash> InstantiateBuilder<T, Args, C, Seal, CodeHash>
where
    T: EnvTypes,
{
    /// Sets the maximum allowed gas costs for the call.
    #[inline]
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.params.gas_limit = gas_limit;
        self
    }

    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn endowment(mut self, value: T::Balance) -> Self {
        self.params.endowment = value;
        self
    }
}

impl<T, Args, C, Seal> InstantiateBuilder<T, Args, C, Seal, state::CodeHashUnassigned>
where
    T: EnvTypes,
{
    /// Using the given code hash.
    #[inline]
    pub fn using_code(
        mut self,
        code_hash: T::Hash,
    ) -> InstantiateBuilder<T, Args, C, Seal, state::CodeHashAssigned> {
        self.params.code_hash = code_hash;
        InstantiateBuilder {
            params: self.params,
            state: Default::default(),
        }
    }
}

impl<T, C, CodeHash>
    InstantiateBuilder<T, EmptyArgumentList, C, state::Unsealed, CodeHash>
where
    T: EnvTypes,
{
    /// Pushes an argument to the inputs of the instantiation.
    #[inline]
    pub fn push_arg<A>(
        self,
        arg: A,
    ) -> InstantiateBuilder<
        T,
        ArgumentList<Argument<A>, EmptyArgumentList>,
        C,
        state::Unsealed,
        CodeHash,
    >
    where
        A: scale::Encode,
    {
        InstantiateBuilder {
            params: InstantiateParams {
                code_hash: self.params.code_hash,
                gas_limit: self.params.gas_limit,
                endowment: self.params.endowment,
                call_data: self.params.call_data.push_arg(arg),
                contract_marker: self.params.contract_marker,
            },
            state: Default::default(),
        }
    }
}

impl<T, ArgsHead, ArgsRest, C, CodeHash>
    InstantiateBuilder<T, ArgsList<ArgsHead, ArgsRest>, C, state::Unsealed, CodeHash>
where
    T: EnvTypes,
{
    /// Pushes an argument to the inputs of the instantiation.
    #[inline]
    pub fn push_arg<A>(
        self,
        arg: A,
    ) -> InstantiateBuilder<
        T,
        ArgsList<A, ArgsList<ArgsHead, ArgsRest>>,
        C,
        state::Unsealed,
        CodeHash,
    >
    where
        A: scale::Encode,
    {
        InstantiateBuilder {
            params: InstantiateParams {
                code_hash: self.params.code_hash,
                gas_limit: self.params.gas_limit,
                endowment: self.params.endowment,
                call_data: self.params.call_data.push_arg(arg),
                contract_marker: self.params.contract_marker,
            },
            state: Default::default(),
        }
    }
}

impl<T, Args, C, CodeHash> InstantiateBuilder<T, Args, C, state::Unsealed, CodeHash>
where
    T: EnvTypes,
{
    /// Seals the create builder to prevent further arguments.
    #[inline]
    pub fn seal(self) -> InstantiateBuilder<T, Args, C, state::Sealed, CodeHash> {
        InstantiateBuilder {
            params: self.params,
            state: Default::default(),
        }
    }
}

impl<T, Args, C, Seal> InstantiateBuilder<T, Args, C, Seal, state::CodeHashAssigned>
where
    T: EnvTypes,
    Args: scale::Encode,
    C: FromAccountId<T>,
{
    /// Instantiates the contract and returns its account ID back to the caller.
    #[inline]
    pub fn instantiate(self) -> Result<C> {
        crate::env::instantiate_contract(&self.params).map(FromAccountId::from_account_id)
    }
}
