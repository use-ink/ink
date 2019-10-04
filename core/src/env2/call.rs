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

//! Infrastructure for calling and instantiating contracts from within contracts.

use crate::{
    env2::{
        errors::{
            CallError,
            CreateError,
        },
        CallParams,
        CreateParams,
        Env,
        EnvTypes,
    },
    memory::{
        vec,
        vec::Vec,
    },
};
use core::marker::PhantomData;
use derive_more::From;
use scale::Decode;

/// The function selector.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From, scale::Decode, scale::Encode)]
pub struct Selector {
    /// The 4 underlying bytes.
    bytes: [u8; 4],
}

impl<'a> From<&'a [u8]> for Selector {
    /// Computes the selector from the given input bytes.
    ///
    /// # Note
    ///
    /// Normally this is invoked through `Selector::from_str`.
    fn from(input: &'a [u8]) -> Self {
        let keccak = ink_utils::hash::keccak256(input);
        Self {
            bytes: [keccak[0], keccak[1], keccak[2], keccak[3]],
        }
    }
}

impl Selector {
    /// Returns the selector for the given name.
    pub fn from_str(name: &str) -> Self {
        From::from(name.as_bytes())
    }

    /// Returns the underlying bytes of the selector.
    pub fn to_bytes(&self) -> [u8; 4] {
        self.bytes
    }
}

/// The raw ABI respecting input data to a call.
///
/// # Note
///
/// The first four bytes are the function selector and the rest are SCALE encoded inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallData {
    /// Already encoded function selector and inputs.
    ///
    /// # Note
    ///
    /// Has the invariant of always holding at least 4 bytes (the selector).
    bytes: Vec<u8>,
}

impl CallData {
    /// Creates new call ABI data for the given selector.
    pub fn new(selector: Selector) -> Self {
        let bytes = selector.to_bytes();
        Self {
            bytes: vec![bytes[0], bytes[1], bytes[2], bytes[3]],
        }
    }

    /// Pushes the given argument onto the call ABI data in encoded form.
    pub fn push_arg<A>(&mut self, arg: &A)
    where
        A: scale::Encode,
    {
        arg.encode_to(&mut self.bytes)
    }

    /// Returns the selector of `self`.
    pub fn selector(&self) -> Selector {
        debug_assert!(self.bytes.len() >= 4);
        let bytes = [self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]];
        bytes.into()
    }

    /// Returns the underlying bytes of the encoded input parameters.
    pub fn params(&self) -> &[u8] {
        debug_assert!(self.bytes.len() >= 4);
        &self.bytes[4..]
    }

    /// Returns the underlying byte representation.
    pub fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl scale::Encode for CallData {
    fn size_hint(&self) -> usize {
        self.bytes.len()
    }

    fn encode_to<T: scale::Output>(&self, dest: &mut T) {
        dest.write(self.bytes.as_slice());
    }
}

impl scale::Decode for CallData {
    fn decode<I: scale::Input>(
        input: &mut I,
    ) -> core::result::Result<Self, scale::Error> {
        let remaining_len = input.remaining_len().unwrap_or(None).unwrap_or(0);
        let mut bytes = Vec::with_capacity(remaining_len);
        while let Ok(byte) = input.read_byte() {
            bytes.push(byte);
        }
        Ok(Self { bytes })
    }
}

/// Represents a return type.
///
/// Used as a marker type to differentiate at compile-time between invoke and evaluate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReturnType<T>(PhantomData<fn() -> T>);

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
    call_data: CallData,
    /// The type of the instantiated contract.
    contract_marker: PhantomData<fn() -> C>,
}

impl<E, C> CreateParams<E> for CreateBuilder<E, C>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    fn code_hash(&self) -> &E::Hash {
        &self.code_hash
    }

    /// The gas limit for the contract instantiation.
    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
    /// The endowment for the instantiated contract.
    fn endowment(&self) -> &E::Balance {
        &self.value
    }

    /// The raw encoded input data.
    fn input_data(&self) -> &CallData {
        &self.call_data
    }
}

impl<E, C> CreateBuilder<E, C>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Creates a new create builder to guide instantiation of a smart contract.
    pub fn new(code_hash: E::Hash, selector: Selector) -> Self {
        Self {
            code_hash,
            gas_limit: 0,
            value: Default::default(),
            call_data: CallData::new(selector),
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
    /// The already encoded call data respecting the ABI.
    call_data: CallData,
}

impl<E, R> CallParams<E> for CallBuilder<E, R>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    fn callee(&self) -> &E::AccountId {
        &self.account_id
    }

    /// The gas limit for the contract instantiation.
    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
    /// The endowment for the instantiated contract.
    fn endowment(&self) -> &E::Balance {
        &self.value
    }

    /// The raw encoded input data.
    fn input_data(&self) -> &CallData {
        &self.call_data
    }
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
        this.call_data.push_arg(arg);
        this
    }
}

/// Types that can be contructed from an `AccountId`
///
/// # Note
///
/// This is needed because of conflicting implementations of `From<T> for T`
/// in the generated code of `ink_lang`.
pub trait FromAccountId<E>
where
    E: EnvTypes,
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
        E::create_contract(&mut Vec::new(), &self)
            .map(FromAccountId::from_account_id)
            .map_err(|_| CreateError)
    }
}

impl<E, R> CallBuilder<E, ReturnType<R>>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Instantiates an evaluatable (returns data) remote smart contract call.
    pub fn eval(account_id: E::AccountId, selector: Selector) -> Self {
        Self {
            account_id,
            gas_limit: 0,
            value: E::Balance::default(),
            return_type: PhantomData,
            call_data: CallData::new(selector),
        }
    }
}

impl<E> CallBuilder<E, ()>
where
    E: EnvTypes,
    E::Balance: Default,
{
    /// Instantiates a non-evaluatable (returns no data) remote smart contract call.
    pub fn invoke(account_id: E::AccountId, selector: Selector) -> Self {
        Self {
            account_id,
            gas_limit: 0,
            value: E::Balance::default(),
            return_type: PhantomData,
            call_data: CallData::new(selector),
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
        this.call_data.push_arg(arg);
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
        E::eval_contract(&mut Vec::new(), &self).map_err(|_| CallError)
    }
}

impl<E> CallBuilder<E, ()>
where
    E: Env,
{
    /// Fires the call to the remote smart contract.
    pub fn fire(self) -> Result<(), CallError> {
        E::invoke_contract(&mut Vec::new(), &self).map_err(|_| CallError)
    }
}
