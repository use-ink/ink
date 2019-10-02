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
        utils::{
            EnlargeTo,
            Reset,
        },
        Result,
    },
    storage::Key,
};
use scale::Codec;

/// The environmental types usable by contracts defined with ink!.
pub trait EnvTypes {
    /// The type of an address.
    type AccountId: 'static + Codec + Clone + PartialEq + Eq;
    /// The type of balances.
    type Balance: 'static + Codec + Clone + PartialEq + Eq;
    /// The type of hash.
    type Hash: 'static + Codec + Clone + PartialEq + Eq;
    /// The type of timestamps.
    type Moment: 'static + Codec + Clone + PartialEq + Eq;
    /// The type of block number.
    type BlockNumber: 'static + Codec + Clone + PartialEq + Eq;
    /// The type of a call into the runtime
    type Call: 'static + scale::Encode;
}

/// Allows reading contract properties.
pub trait GetProperty<P>
where
    P: property::ReadProperty,
{
    /// Gets the property.
    ///
    /// Uses `buffer` for intermediate computation.
    fn get_property<I>(buffer: &mut I) -> P::In
    where
        I: AsMut<[u8]> + EnlargeTo;
}

/// Allows mutating contract properties.
pub trait SetProperty<P>
where
    P: property::WriteProperty,
{
    /// Sets the property.
    ///
    /// Uses `buffer` for intermediate computation.
    fn set_property<O>(buffer: &mut O, encoded: &P::Out)
    where
        O: scale::Output + AsRef<[u8]> + Reset;
}

/// The interface that ink! environments have to implement.
pub trait Env:
    EnvTypes
    + Sized
    + GetProperty<property::Caller<Self>>
    + GetProperty<property::TransferredBalance<Self>>
    + GetProperty<property::GasPrice<Self>>
    + GetProperty<property::GasLeft<Self>>
    + GetProperty<property::NowInMs<Self>>
    + GetProperty<property::Address<Self>>
    + GetProperty<property::Balance<Self>>
    + GetProperty<property::RentAllowance<Self>>
    + SetProperty<property::RentAllowance<Self>>
    + GetProperty<property::BlockNumber<Self>>
    + GetProperty<property::MinimumBalance<Self>>
    + GetProperty<property::Input<Self>>
{
    /// Returns the value at the contract storage at the position of the key.
    ///
    /// # Errors
    ///
    /// - If `key` associates no elements.
    /// - If the element at `key` could not be decoded into `T`.
    fn get_contract_storage<I, T>(buffer: &mut I, key: Key) -> Result<T>
    where
        I: AsMut<[u8]> + EnlargeTo,
        T: scale::Decode;

    /// Sets the value at the key to the given encoded value.
    fn set_contract_storage<O, T>(buffer: &mut O, key: Key, val: &T)
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
        D: CallParams<Self>;

    /// Evaluates a contract call with the given call data.
    ///
    /// # Note
    ///
    /// Evaluations return a return value back to the caller.
    fn eval_contract<IO, D, R>(buffer: &mut IO, call_data: &D) -> Result<R>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode,
        D: CallParams<Self>;

    /// Instantiates a contract from the given create data and returns its account ID.
    fn create_contract<IO, D>(
        buffer: &mut IO,
        create_data: &D,
    ) -> Result<Self::AccountId>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        D: CreateParams<Self>;

    /// Emits an event with the given event data.
    fn emit_event<O, D>(buffer: &mut O, event_data: &D)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        D: EmitEventParams<Self>;

    /// Invokes a runtime dispatchable function with the given call data.
    fn invoke_runtime<O, V>(buffer: &mut O, call_data: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode;

    /// Restores the contract to the given address.
    ///
    /// The `filtered_keys` indicate keys to ignore for evaluating
    /// the equality of the restoration storage. These keys might be required
    /// by the restoring contract but not by the restored contract.
    fn restore_to<O>(
        buffer: &mut O,
        dest: Self::AccountId,
        code_hash: Self::Hash,
        rent_allowance: Self::Balance,
        filtered_keys: &[Key],
    ) where
        O: scale::Output + AsRef<[u8]> + Reset;

    /// Returns the given value back to the caller of the executed contract.
    fn output<O, R>(buffer: &mut O, return_value: &R)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        R: scale::Encode;

    /// Returns a random hash given the subject.
    fn random<I>(buffer: &mut I, subject: &[u8]) -> Self::Hash
    where
        I: AsMut<[u8]> + EnlargeTo;

    /// Prints the contents as a single line.
    ///
    /// # Note
    ///
    /// This is a pure debug utility and should not be used in production.
    /// In fact production chains will generally reject contracts upon deploy
    /// that make use of this functionality.
    fn println(content: &str);
}

/// Types implementing this are suitable as call data.
pub trait CallParams<E>
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
pub trait CreateParams<E>
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
pub trait EmitEventParams<E>
where
    E: EnvTypes,
{
    /// The event topics.
    fn topics(&self) -> &[E::Hash];
    /// The raw encoded event data.
    fn data(&self) -> &[u8];
}
