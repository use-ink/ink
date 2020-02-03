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

use core::marker::PhantomData;
use ink_core::{
    env,
    env::{
        call::{
            CallParams,
            InstantiateParams,
            ReturnType,
        },
        EnvTypes,
        Result,
        Topics,
    },
};
use ink_primitives::Key;

/// Allows to directly access the environment mutably.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait Env {
    /// The environmental types.
    type EnvAccess;

    /// Accesses the environment with predefined environmental types.
    fn env(self) -> Self::EnvAccess;
}

/// A typed accessor to the environment.
///
/// This allows ink! messages to make use of the environment efficiently
/// and user friendly while also maintaining access invariants.
pub struct EnvAccess<'a, T> {
    /// Tricks the Rust compiler into thinking that we use `T`.
    marker: PhantomData<fn() -> &'a T>,
}

impl<'a, T> Default for EnvAccess<'a, T> {
    fn default() -> Self {
        Self {
            marker: Default::default(),
        }
    }
}

impl<'a, E> core::fmt::Debug for EnvAccess<'a, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("EnvAccess").finish()
    }
}

impl<'a, T> EnvAccess<'a, T>
where
    T: EnvTypes,
{
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Panics
    ///
    /// If the returned caller cannot be properly decoded.
    pub fn caller(self) -> T::AccountId {
        env::caller::<T>().expect("couldn't decode caller")
    }

    /// Returns the transferred balance for the contract execution.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn transferred_balance(self) -> T::Balance {
        env::transferred_balance::<T>().expect("couldn't decode transferred balance")
    }

    /// Returns the current price for gas.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn gas_price(self) -> T::Balance {
        env::gas_price::<T>().expect("couldn't decode gas price")
    }

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn gas_left(self) -> T::Balance {
        env::gas_left::<T>().expect("couldn't decode gas left")
    }

    /// Returns the timstamp of the current block.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn block_timestamp(self) -> T::Timestamp {
        env::block_timestamp::<T>().expect("couldn't decode block time stamp")
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn account_id(self) -> T::AccountId {
        env::account_id::<T>().expect("couldn't decode contract account ID")
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Note
    ///
    /// This functionality is deprecated. Please use [`EnvAccess::account_id`]
    /// instead.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    #[deprecated(note = "please use self.env().account_id")]
    pub fn address(self) -> T::AccountId {
        env::account_id::<T>().expect("couldn't decode contract account ID")
    }

    /// Returns the balance of the executed contract.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn balance(self) -> T::Balance {
        env::balance::<T>().expect("couldn't decode contract balance")
    }

    /// Returns the current rent allowance for the executed contract.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn rent_allowance(self) -> T::Balance {
        env::rent_allowance::<T>().expect("couldn't decode contract rent allowance")
    }

    /// Returns the current block number.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn block_number(self) -> T::BlockNumber {
        env::block_number::<T>().expect("couldn't decode block number")
    }

    /// Returns the minimum balance for the contracts chain.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn minimum_balance(self) -> T::Balance {
        env::minimum_balance::<T>().expect("couldn't decode minimum account balance")
    }

    /// Returns the tombstone deposit for the contracts chain.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn tombstone_deposit(self) -> T::Balance {
        env::tombstone_deposit::<T>().expect("couldn't decode tombstone deposits")
    }

    /// Emits an event with the given event data.
    pub fn emit_event<Event>(self, event: Event)
    where
        Event: Topics<T> + scale::Encode,
    {
        env::emit_event::<T, Event>(event)
    }

    /// Sets the rent allowance of the executed contract to the new value.
    pub fn set_rent_allowance(self, new_value: T::Balance) {
        env::set_rent_allowance::<T>(new_value)
    }

    /// Invokes a call to the runtime.
    ///
    /// # Note
    ///
    /// The call is not guaranteed to execute immediately but might be deferred
    /// to the end of the contract execution.
    ///
    /// # Errors
    ///
    /// - If the called runtime function does not exist.
    pub fn invoke_runtime(self, params: &T::Call) -> Result<()> {
        env::invoke_runtime::<T>(params)
    }

    /// Invokes a contract message.
    ///
    /// # Note
    ///
    /// This is a low level way to invoke another smart contract.
    /// Prefer to use the ink! guided and type safe approach to using this.
    ///
    /// # Errors
    ///
    /// - If the called contract does not exist.
    /// - If the called contract is a tombstone.
    /// - If arguments passed to the called contract message are invalid.
    /// - If the called contract execution has trapped.
    /// - If the called contract ran out of gas upon execution.
    pub fn invoke_contract(self, params: &CallParams<T, ()>) -> Result<()> {
        env::invoke_contract::<T>(params)
    }

    /// Evaluates a contract message and returns its result.
    ///
    /// # Note
    ///
    /// This is a low level way to evaluate another smart contract.
    /// Prefer to use the ink! guided and type safe approach to using this.
    ///
    /// # Errors
    ///
    /// - If the called contract does not exist.
    /// - If the called contract is a tombstone.
    /// - If arguments passed to the called contract message are invalid.
    /// - If the called contract execution has trapped.
    /// - If the called contract ran out of gas upon execution.
    /// - If the returned value failed to decode properly.
    pub fn eval_contract<R>(self, params: &CallParams<T, ReturnType<R>>) -> Result<R>
    where
        R: scale::Decode,
    {
        env::eval_contract::<T, R>(params)
    }

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// This is a low level way to instantiate another smart contract.
    /// Prefer to use the ink! guided and type safe approach to using this.
    ///
    /// # Errors
    ///
    /// - If the code hash is invalid.
    /// - If the arguments passed to the instantiation process are invalid.
    /// - If the instantiation process traps.
    /// - If the instantiation process runs out of gas.
    /// - If given insufficient endowment.
    /// - If the returned account ID failed to decode properly.
    pub fn instantiate_contract<C>(
        self,
        params: &InstantiateParams<T, C>,
    ) -> Result<T::AccountId> {
        env::create_contract::<T, C>(params)
    }

    /// Restores a smart contract in tombstone state.
    ///
    /// # Params
    ///
    /// - `account_id`: Account ID of the to-be-restored contract.
    /// - `code_hash`: Code hash of the to-be-restored contract.
    /// - `rent_allowance`: Rent allowance of the restored contract
    ///                     upon successful restoration.
    /// - `filtered_keys`: Storage keys to be excluded when calculating the tombstone hash,
    ///                    which is used to decide whether the original contract and the
    ///                    to-be-restored contract have matching storage.
    ///
    /// # Usage
    ///
    /// A smart contract that has insufficient funds to pay for its storage fees
    /// can eventually be evicted. An evicted smart contract `C` leaves behind
    /// a tombstone associated with a hash that has been computed partially
    /// by its storage contents.
    ///
    /// To restore contract `C` back to a fully working contract the normal
    /// process is to write another contract `C2` with the only purpose to
    /// eventually have the absolutely same contract storage as `C` did when
    /// it was evicted.
    /// For that purpose `C2` can use other storage keys that have not been in
    /// use by contract `C`.
    /// Once `C2` contract storage matches the storage of `C` when it was evicted
    /// `C2` can invoke this method in order to initiate restoration of `C`.
    /// A tombstone hash is calculated for `C2` and if it matches the tombstone
    /// hash of `C` the restoration is going to be successful.
    /// The `filtered_keys` argument can be used to ignore the extraneous keys
    /// used by `C2` but not used by `C`.
    ///
    /// The process of such a smart contract restoration can generally be very expensive.
    ///
    /// # Note
    ///
    /// - The `filtered_keys` can be used to ignore certain storage regions
    ///   in the restorer contract to not influence the hash calculations.
    /// - Does *not* perform restoration right away but defers it to the end of
    ///   the contract execution.
    /// - Restoration is cancelled if there is no tombstone in the destination
    ///   address or if the hashes don't match. No changes are made in this case.
    pub fn restore_contract(
        self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) {
        env::restore_contract::<T>(account_id, code_hash, rent_allowance, filtered_keys)
    }

    /// Returns a random hash.
    ///
    /// # Note
    ///
    /// - The subject buffer can be used to further randomize the hash.
    /// - Within the same execution returns the same random hash for the same subject.
    ///
    /// # Panics
    ///
    /// If the returned value cannot be properly decoded.
    pub fn random(self, subject: &[u8]) -> T::Hash
    where
        T: EnvTypes,
    {
        env::random::<T>(subject).expect("couldn't decode randomized hash")
    }

    /// Returns the value from the *runtime* storage at the position of the key if any.
    ///
    /// # Errors
    ///
    /// - If the key's entry is empty
    /// - If the decoding of the typed value failed
    pub fn get_runtime_storage<R>(self, runtime_key: &[u8]) -> Option<Result<R>>
    where
        R: scale::Decode,
    {
        env::get_runtime_storage::<R>(runtime_key)
    }
}
