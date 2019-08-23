// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

// We need this to fix a rustfmt issue. https://github.com/rust-lang/rustfmt/issues/3750
#[rustfmt::skip]
use crate::{
    env::{
        self,
        CallError,
        Env,
        EnvTypes,
    },
    memory::{
        vec,
        vec::Vec,
    },
};
use core::marker::PhantomData;
use scale::Decode;

/// Consists of the input data to a call.
/// The first four bytes are the function selector and the rest are SCALE encoded inputs.
pub struct CallAbi {
    /// Already encoded function selector and inputs.
    raw: Vec<u8>,
}

impl CallAbi {
    /// Creates new call ABI data for the given selector.
    pub fn new(selector: u32) -> Self {
        let bytes = selector.to_le_bytes();
        Self {
            raw: vec![bytes[0], bytes[1], bytes[2], bytes[3]],
        }
    }

    /// Pushes the given argument onto the call ABI data in encoded form.
    pub fn push_arg<A>(self, arg: &A) -> Self
    where
        A: scale::Encode,
    {
        let mut this = self;
        this.raw.extend(&arg.encode());
        this
    }

    /// Returns the underlying byte representation.
    pub fn to_bytes(&self) -> &[u8] {
        &self.raw
    }
}

/// Represents a return type.
///
/// Used as a marker type to differentiate at compile-time between invoke and evaluate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReturnType<T>(PhantomData<T>);

/// Builds up a call.
pub struct CallBuilder<E, R>
where
    E: EnvTypes,
{
    /// The account ID of the to-be-called smart contract.
    account_id: E::AccountId,
    /// The maximum gas costs allowed for the call.
    gas_cost: u64,
    /// The transferred value for the call.
    value: E::Balance,
    /// The expected return type.
    return_type: PhantomData<ReturnType<R>>,
    /// The already encoded call ABI data.
    raw_input: CallAbi,
}

impl<E, R> CallBuilder<E, ReturnType<R>>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Instantiates an evaluatable (returns data) remote smart contract call.
    pub fn eval(account_id: E::AccountId, selector: u32) -> Self {
        Self {
            account_id,
            gas_cost: 0,
            value: E::Balance::default(),
            return_type: PhantomData,
            raw_input: CallAbi::new(selector),
        }
    }
}

impl<E, R> CallBuilder<E, R>
where
    E: EnvTypes,
{
    /// Sets the maximumly allowed gas costs for the call.
    pub fn gas_cost(self, gas_cost: u64) -> Self {
        let mut this = self;
        this.gas_cost = gas_cost;
        this
    }

    /// Sets the value transferred upon the execution of the call.
    pub fn value(self, value: E::Balance) -> Self {
        let mut this = self;
        this.value = value;
        this
    }

    /// Pushes an argument to the inputs of the call.
    pub fn push_arg<A>(self, arg: &A) -> Self
    where
        A: scale::Encode,
    {
        let mut this = self;
        this.raw_input = this.raw_input.push_arg(arg);
        this
    }
}

impl<E> CallBuilder<E, ()>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Instantiates a non-evaluatable (returns no data) remote smart contract call.
    pub fn invoke(account_id: E::AccountId, selector: u32) -> Self {
        Self {
            account_id,
            gas_cost: 0,
            value: E::Balance::default(),
            return_type: PhantomData,
            raw_input: CallAbi::new(selector),
        }
    }
}

impl<E, R> CallBuilder<E, ReturnType<R>>
where
    E: Env,
    R: Decode,
{
    /// Fires the call to the remote smart contract.
    /// Returns the returned data back to the caller.
    pub fn fire(self) -> Result<R, CallError> {
        env::call_evaluate::<E, R>(
            self.account_id,
            self.gas_cost,
            self.value,
            self.raw_input.to_bytes(),
        )
    }
}

impl<E> CallBuilder<E, ()>
where
    E: Env,
{
    /// Fires the call to the remote smart contract.
    pub fn fire(self) -> Result<(), CallError> {
        env::call_invoke::<E>(
            self.account_id,
            self.gas_cost,
            self.value,
            self.raw_input.to_bytes(),
        )
    }
}
