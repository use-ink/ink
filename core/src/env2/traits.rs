// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use scale::Codec;

use crate::{
    env2::{
        call::{
            CallParams,
            CreateParams,
            ReturnType,
        },
        property,
        utils::{
            EnlargeTo,
            Reset,
        },
        Result,
    },
    storage::Key,
};

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
    /// Invocations fire and forget and thus won't return a value back.
    fn invoke_contract<O>(buffer: &mut O, call_data: &CallParams<Self, ()>) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset;

    /// Evaluates a contract call with the given call data.
    ///
    /// # Note
    ///
    /// Evaluations return a return value back to the caller.
    fn eval_contract<IO, R>(
        buffer: &mut IO,
        call_data: &CallParams<Self, ReturnType<R>>,
    ) -> Result<R>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode;

    /// Instantiates a contract from the given create data and returns its account ID.
    fn create_contract<IO, C>(
        buffer: &mut IO,
        create_data: &CreateParams<Self, C>,
    ) -> Result<Self::AccountId>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset;

    /// Emits an event with the given event data.
    fn emit_event<O, Event>(buffer: &mut O, event_data: Event)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        Event: Topics<Self> + scale::Encode;

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

    /// Returns the value from the *runtime* storage at the position of the key.
    ///
    /// # Errors
    ///
    /// - If `key` associates no elements.
    /// - If the element at `key` could not be decoded into `T`.
    fn get_runtime_storage<I, T>(buffer: &mut I, key: &[u8]) -> Result<T>
    where
        I: AsMut<[u8]> + EnlargeTo,
        T: scale::Decode;
}

/// Implemented by event types to communicate their topic hashes.
pub trait Topics<E>
where
    E: EnvTypes,
{
    /// Returns the topic hashes of `self`.
    fn topics(&self) -> &'static [<E as EnvTypes>::Hash];
}
