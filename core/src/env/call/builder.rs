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
        CallData,
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
pub struct CallParams<E, R>
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
    call_data: CallData,
}

/// Builds up a call.
pub struct CallBuilder<E, R, Seal>
where
    E: EnvTypes,
{
    /// The current parameters that have been built up so far.
    params: CallParams<E, R>,
    /// Seal state.
    seal: PhantomData<Seal>,
}

impl<E, R> CallParams<E, R>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    pub fn callee(&self) -> &E::AccountId {
        &self.callee
    }

    /// The gas limit for the contract instantiation.
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
    /// The transferred value for the called contract.
    pub fn transferred_value(&self) -> &E::Balance {
        &self.transferred_value
    }

    /// The raw encoded input data.
    pub fn input_data(&self) -> &CallData {
        &self.call_data
    }
}

impl<E, R> CallParams<E, R>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Creates the default set of parameters for the cross-contract call.
    fn new(callee: E::AccountId, selector: Selector) -> Self {
        Self {
            callee,
            gas_limit: 0,
            transferred_value: E::Balance::default(),
            return_type: PhantomData,
            call_data: CallData::new(selector),
        }
    }

    /// Returns a builder for a cross-contract call that might return data.
    pub fn eval(
        callee: E::AccountId,
        selector: Selector,
    ) -> CallBuilder<E, ReturnType<R>, state::Unsealed> {
        CallBuilder {
            params: CallParams::new(callee, selector),
            seal: Default::default(),
        }
    }

    /// Returns a builder for a cross-contract call that cannot return data.
    ///
    /// Prefer this over [`CallParams::eval`] if possible since it is the more efficient operation.
    pub fn invoke(
        callee: E::AccountId,
        selector: Selector,
    ) -> CallBuilder<E, (), state::Unsealed> {
        CallBuilder {
            params: CallParams::new(callee, selector),
            seal: Default::default(),
        }
    }
}

impl<E, R, Seal> CallBuilder<E, R, Seal>
where
    E: EnvTypes,
{
    /// Sets the maximumly allowed gas costs for the call.
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.params.gas_limit = gas_limit;
        self
    }

    /// Sets the value transferred upon the execution of the call.
    pub fn transferred_value(mut self, value: E::Balance) -> Self {
        self.params.transferred_value = value;
        self
    }
}

impl<E, R> CallBuilder<E, R, state::Unsealed>
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

    /// Seals the call builder to prevent further arguments.
    pub fn seal(self) -> CallBuilder<E, R, state::Sealed> {
        CallBuilder {
            params: self.params,
            seal: Default::default(),
        }
    }
}

impl<E, R, Seal> CallBuilder<E, ReturnType<R>, Seal>
where
    E: EnvTypes,
    R: scale::Decode,
{
    /// Fires the call to the remote smart contract.
    /// Returns the returned data back to the caller.
    pub fn fire(self) -> Result<R>
    where
        R: scale::Decode,
    {
        crate::env::eval_contract(&self.params)
    }
}

impl<E, Seal> CallBuilder<E, (), Seal>
where
    E: EnvTypes,
{
    /// Fires the cross-call to the smart contract.
    pub fn fire(self) -> Result<()> {
        crate::env::invoke_contract(&self.params)
    }
}
