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

use crate::{
    env,
    env::{
        call::{
            utils::{
                EmptyArgumentList,
                ReturnType,
                Set,
                Unset,
                Unwrap,
            },
            ExecutionInput,
        },
        EnvTypes,
    },
};
use core::marker::PhantomData;

/// Contracts that can be contructed from an `AccountId`.
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
#[derive(Debug)]
pub struct CreateParams<E, Args, R>
where
    E: EnvTypes,
{
    /// The code hash of the created contract.
    code_hash: E::Hash,
    /// The maximum gas costs allowed for the instantiation.
    gas_limit: u64,
    /// The endowment for the instantiated contract.
    endowment: E::Balance,
    /// The input data for the instantation.
    exec_input: ExecutionInput<Args>,
    /// The type of the instantiated contract.
    return_type: ReturnType<R>,
}

/// Builds up contract instantiations.
pub struct CreateBuilder<E, CodeHash, GasLimit, Endowment, Args, R>
where
    E: EnvTypes,
{
    env_types: PhantomData<fn() -> E>,
    code_hash: CodeHash,
    gas_limit: GasLimit,
    endowment: Endowment,
    exec_input: Args,
    return_type: ReturnType<R>,
}

/// Returns a new [`CreateBuilder`] to build up the parameters to a cross-contract instantiation.
pub fn build_create<E, R>() -> CreateBuilder<
    E,
    Unset<E::Hash>,
    Unset<u64>,
    Unset<E::Balance>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    R,
>
where
    E: EnvTypes,
    R: FromAccountId<E>,
{
    CreateBuilder {
        env_types: Default::default(),
        code_hash: Default::default(),
        gas_limit: Default::default(),
        endowment: Default::default(),
        exec_input: Default::default(),
        return_type: Default::default(),
    }
}

#[cfg(
    // We do not currently support cross-contract instantiation in the off-chain
    // environment so we do not have to provide these getters in case of
    // off-chain environment compilation.
    all(not(feature = "std"), target_arch = "wasm32")
)]
impl<E, Args, R> CreateParams<E, Args, R>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    #[inline]
    pub(crate) fn code_hash(&self) -> &E::Hash {
        &self.code_hash
    }

    /// The gas limit for the contract instantiation.
    #[inline]
    pub(crate) fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    /// The endowment for the instantiated contract.
    #[inline]
    pub(crate) fn endowment(&self) -> &E::Balance {
        &self.endowment
    }

    /// The raw encoded input data.
    #[inline]
    pub(crate) fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

impl<E, Args, R> CreateParams<E, Args, R>
where
    E: EnvTypes,
    Args: scale::Encode,
    R: FromAccountId<E>,
{
    /// Instantiates the contract and returns its account ID back to the caller.
    #[inline]
    pub fn instantiate(&self) -> Result<R, env::EnvError> {
        env::instantiate_contract(self).map(FromAccountId::from_account_id)
    }
}

impl<E, GasLimit, Endowment, Args, R>
    CreateBuilder<E, Unset<E::Hash>, GasLimit, Endowment, Args, R>
where
    E: EnvTypes,
{
    /// Sets the used code hash for the contract instantiation.
    #[inline]
    pub fn code_hash(
        self,
        code_hash: E::Hash,
    ) -> CreateBuilder<E, Set<E::Hash>, GasLimit, Endowment, Args, R> {
        CreateBuilder {
            env_types: Default::default(),
            code_hash: Set(code_hash),
            gas_limit: self.gas_limit,
            endowment: self.endowment,
            exec_input: self.exec_input,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, Endowment, Args, R>
    CreateBuilder<E, CodeHash, Unset<u64>, Endowment, Args, R>
where
    E: EnvTypes,
{
    /// Sets the maximum allowed gas costs for the contract instantiation.
    #[inline]
    pub fn gas_limit(
        self,
        gas_limit: u64,
    ) -> CreateBuilder<E, CodeHash, Set<u64>, Endowment, Args, R> {
        CreateBuilder {
            env_types: Default::default(),
            code_hash: self.code_hash,
            gas_limit: Set(gas_limit),
            endowment: self.endowment,
            exec_input: self.exec_input,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, GasLimit, Args, R>
    CreateBuilder<E, CodeHash, GasLimit, Unset<E::Balance>, Args, R>
where
    E: EnvTypes,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn endowment(
        self,
        endowment: E::Balance,
    ) -> CreateBuilder<E, CodeHash, GasLimit, Set<E::Balance>, Args, R> {
        CreateBuilder {
            env_types: Default::default(),
            code_hash: self.code_hash,
            gas_limit: self.gas_limit,
            endowment: Set(endowment),
            exec_input: self.exec_input,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, GasLimit, Endowment, R>
    CreateBuilder<
        E,
        CodeHash,
        GasLimit,
        Endowment,
        Unset<ExecutionInput<EmptyArgumentList>>,
        R,
    >
where
    E: EnvTypes,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args>,
    ) -> CreateBuilder<E, CodeHash, GasLimit, Endowment, Set<ExecutionInput<Args>>, R>
    {
        CreateBuilder {
            env_types: Default::default(),
            code_hash: self.code_hash,
            gas_limit: self.gas_limit,
            endowment: self.endowment,
            exec_input: Set(exec_input),
            return_type: self.return_type,
        }
    }
}

impl<E, GasLimit, Args, R>
    CreateBuilder<
        E,
        Set<E::Hash>,
        GasLimit,
        Set<E::Balance>,
        Set<ExecutionInput<Args>>,
        R,
    >
where
    E: EnvTypes,
    GasLimit: Unwrap<Output = u64>,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn params(self) -> CreateParams<E, Args, R> {
        CreateParams {
            code_hash: self.code_hash.value(),
            gas_limit: self.gas_limit.unwrap_or_else(|| 0),
            endowment: self.endowment.value(),
            exec_input: self.exec_input.value(),
            return_type: self.return_type,
        }
    }
}
