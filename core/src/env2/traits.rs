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

use crate::{
    env2::{
        call::CallData,
        property,
        EnlargeTo,
        Reset,
        Result,
    },
    storage::Key,
};
use scale::Codec;

/// The environmental types usable by contracts defined with ink!.
pub trait EnvTypes {
    /// The type of an address.
    type AccountId: Codec + Clone + PartialEq + Eq;
    /// The type of balances.
    type Balance: Codec + Clone + PartialEq + Eq;
    /// The type of hash.
    type Hash: Codec + Clone + PartialEq + Eq;
    /// The type of timestamps.
    type Moment: Codec + Clone + PartialEq + Eq;
    /// The type of block number.
    type BlockNumber: Codec + Clone + PartialEq + Eq;
    /// The type of a call into the runtime
    type Call: scale::Encode;
}

/// Allows reading contract properties.
pub trait GetProperty<P>
where
    P: property::ReadProperty,
{
    /// Gets the property.
    ///
    /// Uses `buffer` for intermediate computation.
    fn get_property<I>(buffer: &mut I) -> Result<P::In>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo;
}

/// Allows mutating contract properties.
pub trait SetProperty<P>
where
    P: property::WriteProperty,
{
    /// Sets the property.
    ///
    /// Uses `buffer` for intermediate computation.
    fn set_property<O>(buffer: &mut O, encoded: &P::Out) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset;
}

pub trait Env:
    EnvTypes
    + Sized
    + GetProperty<property::Caller<Self>>
    + GetProperty<property::TransferredBalance<Self>>
    + GetProperty<property::GasPrice<Self>>
    + GetProperty<property::GasLeft<Self>>
    + GetProperty<property::NowInMs<Self>>
    + GetProperty<property::AccountId<Self>>
    + GetProperty<property::Balance<Self>>
    + GetProperty<property::RentAllowance<Self>>
    + SetProperty<property::RentAllowance<Self>>
    + GetProperty<property::BlockNumber<Self>>
    + GetProperty<property::MinimumBalance<Self>>
    + GetProperty<property::Input<Self>>
    + SetProperty<property::Output<Self>>
{
    /// Returns the value at the contract storage at the position of the key.
    ///
    /// # Errors
    ///
    /// - If `key` associates no elements.
    /// - If the element at `key` could not be decoded into `T`.
    fn get_contract_storage<I, T>(key: Key, buffer: &mut I) -> Result<T>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
        T: scale::Decode;

    /// Sets the value at the key to the given encoded value.
    fn set_contract_storage<O, T>(key: Key, buffer: &mut O, val: &T)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        T: scale::Encode;

    /// Clears the value at the key position.
    fn clear_contract_storage(key: Key);

    /// Invokes a contract call with the given call data.
    ///
    /// # Note
    ///
    /// Invokations fire and forget and thus won't return a value back.
    fn invoke_contract<O, D>(buffer: &mut O, call_data: &D) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        D: BuildCall<Self>;

    /// Evaluates a contract call with the given call data.
    ///
    /// # Note
    ///
    /// Evaluations return a return value back to the caller.
    fn eval_contract<IO, D, R>(buffer: &mut IO, call_data: &D) -> Result<R>
    where
        IO: scale::Input + scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode,
        D: BuildCall<Self>;

    /// Instantiates a contract from the given create data and returns its account ID.
    fn create_contract<IO, D>(buffer: &mut IO, create_data: &D) -> Result<Self::AccountId>
    where
        IO: scale::Input + scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        D: BuildCreate<Self>;

    /// Emits an event with the given event data.
    fn emit_event<O, D>(buffer: &mut O, event_data: &D)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        D: BuildEvent<Self>;

    /// Invokes a runtime dispatchable function with the given call data.
    fn invoke_runtime<T>(call_data: &T) -> Result<()>
    where
        T: scale::Encode;

    /// Returns a random hash given the subject.
    fn random<I>(buffer: I, subject: &[u8]) -> Result<Self::Hash>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo;

    /// Prints the contents as a single line.
    fn println(content: &str);
}

/// Types implementing this are suitable as call data.
pub trait BuildCall<E>
where
    E: EnvTypes,
{
    /// The callee of the call.
    fn callee(&self) -> &E::AccountId;
    /// The gas limit for the contract execution.
    fn gas_limit(&self) -> u64;
    /// The endowment for the called contract.
    fn endowment(&self) -> &E::Balance;
    /// The raw encoded input data.
    fn input_data(&self) -> &CallData;
}

/// Types implementing this are suitable as create data.
pub trait BuildCreate<E>
where
    E: EnvTypes,
{
    /// The code hash of the contract.
    fn code_hash(&self) -> &E::Hash;
    /// The gas limit for the contract instantiation.
    fn gas_limit(&self) -> u64;
    /// The endowment for the instantiated contract.
    fn endowment(&self) -> &E::Balance;
    /// The raw encoded input data.
    fn input_data(&self) -> &CallData;
}

/// Types implementing this are suitable as event data.
pub trait BuildEvent<E>
where
    E: EnvTypes,
{
    /// The event topics.
    fn topics(&self) -> &[E::Hash];
    /// The raw encoded event data.
    fn data(&self) -> &[u8];
}
