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

#[rustfmt::skip]
use crate::{
    env::{
        self,
        CallError,
        CreateError,
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
struct CallAbi {
    /// Already encoded function selector and inputs.
    raw: Vec<u8>,
}

impl CallAbi {
    /// Creates new call ABI data for the given selector.
    pub fn new(selector: [u8; 4]) -> Self {
        Self {
            raw: vec![selector[0], selector[1], selector[2], selector[3]],
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

/// Builds up contract instantiations.
pub struct CreateBuilder<E, C>
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
    raw_input: Vec<u8>,
    /// The type of the instantiated contract.
    contract_marker: PhantomData<fn() -> C>,
}

impl<E, C> CreateBuilder<E, C>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Creates a new create builder to guide instantiation of a smart contract.
    pub fn new(code_hash: E::Hash) -> Self {
        Self {
            code_hash,
            gas_limit: 0,
            value: Default::default(),
            raw_input: Vec::new(),
            contract_marker: Default::default(),
        }
    }
}

/// Builds up a call.
pub struct CallBuilder<E, R>
where
    E: EnvTypes,
{
    /// The account ID of the to-be-called smart contract.
    account_id: E::AccountId,
    /// The maximum gas costs allowed for the call.
    gas_limit: u64,
    /// The transferred value for the call.
    value: E::Balance,
    /// The expected return type.
    return_type: PhantomData<ReturnType<R>>,
    /// The already encoded call ABI data.
    raw_input: CallAbi,
}

impl<E, C> CreateBuilder<E, C>
where
    E: EnvTypes,
{
    /// Sets the maximumly allowed gas costs for the call.
    pub fn gas_limit(self, gas_limit: u64) -> Self {
        let mut this = self;
        this.gas_limit = gas_limit;
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
        this.raw_input.extend(&arg.encode());
        this
    }
}

/// Needed because of conflicting implementations of From<T> for T
/// resulting of generated `ink_lang` code.
pub trait FromAccountId<E>
where
    E: Env,
{
    fn from_account_id(account_id: <E as EnvTypes>::AccountId) -> Self;
}

impl<E, C> CreateBuilder<E, C>
where
    E: Env,
    C: FromAccountId<E>,
{
    /// Runs the process to create and instantiate a new smart contract.
    /// Returns the account ID of the newly created smart contract.
    pub fn create(self) -> Result<C, CreateError> {
        env::create::<E>(self.code_hash, self.gas_limit, self.value, &self.raw_input)
            .map(FromAccountId::from_account_id)
    }
}

impl<E, R> CallBuilder<E, ReturnType<R>>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Instantiates an evaluatable (returns data) remote smart contract call.
    pub fn eval(account_id: E::AccountId, selector: [u8; 4]) -> Self {
        Self {
            account_id,
            gas_limit: 0,
            value: E::Balance::default(),
            return_type: PhantomData,
            raw_input: CallAbi::new(selector),
        }
    }
}

impl<E> CallBuilder<E, ()>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Instantiates a non-evaluatable (returns no data) remote smart contract call.
    pub fn invoke(account_id: E::AccountId, selector: [u8; 4]) -> Self {
        Self {
            account_id,
            gas_limit: 0,
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
    pub fn gas_limit(self, gas_limit: u64) -> Self {
        let mut this = self;
        this.gas_limit = gas_limit;
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
            self.gas_limit,
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
            self.gas_limit,
            self.value,
            self.raw_input.to_bytes(),
        )
    }
}
