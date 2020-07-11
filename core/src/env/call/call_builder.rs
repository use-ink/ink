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

/// The final parameters to the cross-contract call.
#[derive(Debug)]
pub struct Call<E, Args, R>
where
    E: EnvTypes,
{
    /// The account ID of the to-be-called smart contract.
    callee: E::AccountId,
    /// The maximum gas costs allowed for the call.
    gas_limit: u64,
    /// The transferred value for the call.
    transferred_value: E::Balance,
    /// The expected return type.
    return_type: ReturnType<R>,
    /// The inputs to the execution which is a selector and encoded arguments.
    exec_input: ExecutionInput<Args>,
}

impl<E, Args, R> Call<E, Args, R>
where
    E: EnvTypes,
{
    /// Returns the account ID of the called contract instance.
    #[inline]
    pub fn callee(&self) -> &E::AccountId {
        &self.callee
    }

    /// Returns the chosen gas limit for the called contract execution.
    #[inline]
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    /// Returns the transferred value for the called contract.
    #[inline]
    pub fn transferred_value(&self) -> &E::Balance {
        &self.transferred_value
    }

    /// Returns the execution input.
    #[inline]
    pub fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

impl<E> Call<E, EmptyArgumentList, ()>
where
    E: EnvTypes,
    E::Balance: Default,
    E::AccountId: Default,
{
    /// Creates the default set of parameters for the cross-contract call.
    pub fn build() -> CallBuilder<
        E,
        Unset<E::AccountId>,
        Unset<u64>,
        Unset<E::Balance>,
        Unset<ExecutionInput<EmptyArgumentList>>,
    > {
        CallBuilder {
            env_types: Default::default(),
            callee: Default::default(),
            gas_limit: Default::default(),
            transferred_value: Default::default(),
            exec_input: Default::default(),
        }
    }
}

/// Builds up a cross contract call.
pub struct CallBuilder<E, Callee, GasLimit, TransferredValue, Args>
where
    E: EnvTypes,
{
    env_types: PhantomData<fn() -> E>,
    /// The current parameters that have been built up so far.
    callee: Callee,
    gas_limit: GasLimit,
    transferred_value: TransferredValue,
    exec_input: Args,
}

impl<E, GasLimit, TransferredValue, Args>
    CallBuilder<E, Unset<E::AccountId>, GasLimit, TransferredValue, Args>
where
    E: EnvTypes,
{
    /// Sets the called smart contract instance account ID to the given value.
    #[inline]
    pub fn callee(
        self,
        callee: E::AccountId,
    ) -> CallBuilder<E, Set<E::AccountId>, GasLimit, TransferredValue, Args> {
        CallBuilder {
            env_types: Default::default(),
            callee: Set(callee),
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
            exec_input: self.exec_input,
        }
    }
}

impl<E, Callee, TransferredValue, Args>
    CallBuilder<E, Callee, Unset<u64>, TransferredValue, Args>
where
    E: EnvTypes,
{
    /// Sets the maximumly allowed gas costs for the call.
    #[inline]
    pub fn gas_limit(
        self,
        gas_limit: u64,
    ) -> CallBuilder<E, Callee, Set<u64>, TransferredValue, Args> {
        CallBuilder {
            env_types: Default::default(),
            callee: self.callee,
            gas_limit: Set(gas_limit),
            transferred_value: self.transferred_value,
            exec_input: self.exec_input,
        }
    }
}

impl<E, Callee, GasLimit, Args> CallBuilder<E, Callee, GasLimit, Unset<E::Balance>, Args>
where
    E: EnvTypes,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn transferred_value(
        self,
        transferred_value: E::Balance,
    ) -> CallBuilder<E, Callee, GasLimit, Set<E::Balance>, Args> {
        CallBuilder {
            env_types: Default::default(),
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value: Set(transferred_value),
            exec_input: self.exec_input,
        }
    }
}

impl<E, Callee, GasLimit, TransferredValue>
    CallBuilder<
        E,
        Callee,
        GasLimit,
        TransferredValue,
        Unset<ExecutionInput<EmptyArgumentList>>,
    >
where
    E: EnvTypes,
{
    /// Sets the execution input to the given value.
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args>,
    ) -> CallBuilder<E, Callee, GasLimit, TransferredValue, Set<ExecutionInput<Args>>>
    {
        CallBuilder {
            env_types: Default::default(),
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
            exec_input: Set(exec_input),
        }
    }
}

impl<E, GasLimit, TransferredValue, Args>
    CallBuilder<
        E,
        Set<E::AccountId>,
        GasLimit,
        TransferredValue,
        Set<ExecutionInput<Args>>,
    >
where
    E: EnvTypes,
    GasLimit: Unwrap<Output = u64>,
    TransferredValue: Unwrap<Output = E::Balance>,
{
    /// Finalizes the call builder to call a function without return value.
    pub fn invoke_params(self) -> Call<E, Args, ()> {
        Call {
            callee: self.callee.value(),
            gas_limit: self.gas_limit.unwrap_or_else(|| 0),
            transferred_value: self
                .transferred_value
                .unwrap_or_else(|| E::Balance::from(0)),
            return_type: Default::default(),
            exec_input: self.exec_input.value(),
        }
    }

    /// Invokes the contract with the given built-up call parameters.
    ///
    /// # Note
    ///
    /// Prefer [`invoke`](`Self::invoke`) over [`eval`](`Self::eval`) if the
    /// called contract message does not return anything because it is more efficient.
    pub fn invoke(self) -> Result<(), env::EnvError>
    where
        Args: scale::Encode,
    {
        env::invoke_contract(&self.invoke_params())
    }

    /// Finalizes the call builder to call a function with the given return value type.
    pub fn eval_params<R>(self) -> Call<E, Args, ReturnType<R>>
    where
        R: scale::Decode,
    {
        Call {
            callee: self.callee.value(),
            gas_limit: self.gas_limit.unwrap_or_else(|| 0),
            transferred_value: self
                .transferred_value
                .unwrap_or_else(|| E::Balance::from(0)),
            return_type: Default::default(),
            exec_input: self.exec_input.value(),
        }
    }

    /// Evaluates the contract with the given built-up call parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Note
    ///
    /// Prefer [`invoke`](`Self::invoke`) over [`eval`](`Self::eval`) if the
    /// called contract message does not return anything because it is more efficient.
    pub fn eval<R>(self) -> Result<R, env::EnvError>
    where
        Args: scale::Encode,
        R: scale::Decode,
    {
        env::eval_contract(&self.eval_params())
    }
}
