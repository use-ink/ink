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
        state,
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

/// Represents a return type.
///
/// Used as a marker type to differentiate at compile-time between invoke and evaluate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReturnType<T>(PhantomData<fn() -> T>);

/// The final parameters to the cross-contract call.
pub struct CallParams<E, Args, R>
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
    return_type: PhantomData<ReturnType<R>>,
    /// The already encoded call data respecting the ABI.
    call_data: ExecutionInput<Args>,
}

/// Builds up a call.
pub struct CallBuilder<E, Args, R, Seal>
where
    E: EnvTypes,
{
    /// The current parameters that have been built up so far.
    params: CallParams<E, Args, R>,
    /// Seal state.
    seal: PhantomData<Seal>,
}

impl<E, Args, R> CallParams<E, Args, R>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    #[inline]
    pub fn callee(&self) -> &E::AccountId {
        &self.callee
    }

    /// The gas limit for the contract instantiation.
    #[inline]
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
    /// The transferred value for the called contract.
    #[inline]
    pub fn transferred_value(&self) -> &E::Balance {
        &self.transferred_value
    }

    /// The raw encoded input data.
    #[inline]
    pub fn input_data(&self) -> &ExecutionInput<Args> {
        &self.call_data
    }
}

impl<E, R> CallParams<E, EmptyArgumentList, R>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Creates the default set of parameters for the cross-contract call.
    #[inline]
    fn new(callee: E::AccountId, selector: Selector) -> Self {
        Self {
            callee,
            gas_limit: 0,
            transferred_value: E::Balance::default(),
            return_type: PhantomData,
            call_data: ExecutionInput::new(selector),
        }
    }

    /// Returns a builder for a cross-contract call that might return data.
    #[inline]
    pub fn eval(
        callee: E::AccountId,
        selector: Selector,
    ) -> CallBuilder<E, EmptyArgumentList, ReturnType<R>, state::Unsealed> {
        CallBuilder {
            params: CallParams::new(callee, selector),
            seal: Default::default(),
        }
    }

    /// Returns a builder for a cross-contract call that cannot return data.
    ///
    /// Prefer this over [`CallParams::eval`] if possible since it is the more efficient operation.
    #[inline]
    pub fn invoke(
        callee: E::AccountId,
        selector: Selector,
    ) -> CallBuilder<E, EmptyArgumentList, (), state::Unsealed> {
        CallBuilder {
            params: CallParams::new(callee, selector),
            seal: Default::default(),
        }
    }
}

impl<E, Args, R, Seal> CallBuilder<E, Args, R, Seal>
where
    E: EnvTypes,
{
    /// Sets the maximumly allowed gas costs for the call.
    #[inline]
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.params.gas_limit = gas_limit;
        self
    }

    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn transferred_value(mut self, value: E::Balance) -> Self {
        self.params.transferred_value = value;
        self
    }
}

impl<E, R> CallBuilder<E, EmptyArgumentList, R, state::Unsealed>
where
    E: EnvTypes,
{
    /// Pushes an argument to the inputs of the call.
    #[inline]
    pub fn push_arg<A>(
        self,
        arg: A,
    ) -> CallBuilder<E, ArgumentList<Argument<A>, EmptyArgumentList>, R, state::Unsealed>
    where
        A: scale::Encode,
    {
        CallBuilder {
            params: CallParams {
                call_data: self.params.call_data.push_arg(arg),
                callee: self.params.callee,
                gas_limit: self.params.gas_limit,
                transferred_value: self.params.transferred_value,
                return_type: self.params.return_type,
            },
            seal: Default::default(),
        }
    }
}

impl<'a, E, ArgsHead, ArgsRest, R>
    CallBuilder<E, ArgsList<ArgsHead, ArgsRest>, R, state::Unsealed>
where
    E: EnvTypes,
{
    /// Pushes an argument to the inputs of the call.
    #[inline]
    pub fn push_arg<A>(
        self,
        arg: A,
    ) -> CallBuilder<E, ArgsList<A, ArgsList<ArgsHead, ArgsRest>>, R, state::Unsealed>
    where
        A: scale::Encode,
    {
        CallBuilder {
            params: CallParams {
                call_data: self.params.call_data.push_arg(arg),
                callee: self.params.callee,
                gas_limit: self.params.gas_limit,
                transferred_value: self.params.transferred_value,
                return_type: self.params.return_type,
            },
            seal: Default::default(),
        }
    }
}

impl<E, Args, R> CallBuilder<E, Args, R, state::Unsealed>
where
    E: EnvTypes,
{
    /// Seals the call builder to prevent further arguments.
    #[inline]
    pub fn seal(self) -> CallBuilder<E, Args, R, state::Sealed> {
        CallBuilder {
            params: self.params,
            seal: Default::default(),
        }
    }
}

impl<E, Args, R, Seal> CallBuilder<E, Args, ReturnType<R>, Seal>
where
    E: EnvTypes,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Fires the call to the remote smart contract.
    /// Returns the returned data back to the caller.
    #[inline]
    pub fn fire(self) -> Result<R>
    where
        R: scale::Decode,
    {
        crate::env::eval_contract(&self.params)
    }
}

impl<E, Args, Seal> CallBuilder<E, Args, (), Seal>
where
    E: EnvTypes,
    Args: scale::Encode,
{
    /// Fires the cross-call to the smart contract.
    #[inline]
    pub fn fire(self) -> Result<()> {
        crate::env::invoke_contract(&self.params)
    }
}
