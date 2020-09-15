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
            utils::ReturnType,
            CallParams,
            CreateParams,
        },
        EnvTypes,
        Result,
    },
};
use ink_primitives::Key;

/// Simplifies interaction with the host environment via `self`.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait Env {
    /// The access wrapper.
    type EnvAccess;

    /// Accesses the environment with predefined environmental types.
    fn env(self) -> Self::EnvAccess;
}

/// Simplifies interaction with the host environment via `Self`.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait StaticEnv {
    /// The access wrapper.
    type EnvAccess;

    /// Accesses the environment with predefined environmental types.
    fn env() -> Self::EnvAccess;
}

/// A typed accessor to the environment.
///
/// This allows ink! messages to make use of the environment efficiently
/// and user friendly while also maintaining access invariants.
#[derive(Copy, Clone)]
pub struct EnvAccess<'a, T> {
    /// Tricks the Rust compiler into thinking that we use `T`.
    marker: PhantomData<fn() -> &'a T>,
}

impl<'a, T> Default for EnvAccess<'a, T> {
    #[inline]
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
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::caller`]
    pub fn caller(self) -> T::AccountId {
        env::caller::<T>().expect("couldn't decode caller")
    }

    /// Returns the transferred balance for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::transferred_balance`]
    pub fn transferred_balance(self) -> T::Balance {
        env::transferred_balance::<T>().expect("couldn't decode transferred balance")
    }

    /// Returns the price for the specified amount of gas.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::gas_price`]
    pub fn weight_to_fee(self, gas: u64) -> T::Balance {
        env::weight_to_fee::<T>(gas).expect("couldn't decode weight fee")
    }

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::gas_left`]
    pub fn gas_left(self) -> T::Balance {
        env::gas_left::<T>().expect("couldn't decode gas left")
    }

    /// Returns the timstamp of the current block.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::block_timestamp`]
    pub fn block_timestamp(self) -> T::Timestamp {
        env::block_timestamp::<T>().expect("couldn't decode block time stamp")
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::account_id`]
    pub fn account_id(self) -> T::AccountId {
        env::account_id::<T>().expect("couldn't decode contract account ID")
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Note
    ///
    /// - This functionality is deprecated. Please use [`EnvAccess::account_id`]
    ///   instead.
    /// - For more details visit: [`ink_core::env::account_id`]
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
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::balance`]
    pub fn balance(self) -> T::Balance {
        env::balance::<T>().expect("couldn't decode contract balance")
    }

    /// Returns the current rent allowance for the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::rent_allowance`]
    pub fn rent_allowance(self) -> T::Balance {
        env::rent_allowance::<T>().expect("couldn't decode contract rent allowance")
    }

    /// Returns the current block number.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::block_number`]
    pub fn block_number(self) -> T::BlockNumber {
        env::block_number::<T>().expect("couldn't decode block number")
    }

    /// Returns the minimum balance for the contracts chain.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::minimum_balance`]
    pub fn minimum_balance(self) -> T::Balance {
        env::minimum_balance::<T>().expect("couldn't decode minimum account balance")
    }

    /// Returns the tombstone deposit for the contracts chain.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::tombstone_deposit`]
    pub fn tombstone_deposit(self) -> T::Balance {
        env::tombstone_deposit::<T>().expect("couldn't decode tombstone deposits")
    }

    /// Sets the rent allowance of the executed contract to the new value.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::set_rent_allowance`]
    pub fn set_rent_allowance(self, new_value: T::Balance) {
        env::set_rent_allowance::<T>(new_value)
    }

    /// Invokes a contract message.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::invoke_contract`]
    pub fn invoke_contract<Args>(self, params: &CallParams<T, Args, ()>) -> Result<()>
    where
        Args: scale::Encode,
    {
        env::invoke_contract::<T, Args>(params)
    }

    /// Evaluates a contract message and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::eval_contract`]
    pub fn eval_contract<Args, R>(
        self,
        params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        Args: scale::Encode,
        R: scale::Decode,
    {
        env::eval_contract::<T, Args, R>(params)
    }

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::instantiate_contract`]
    pub fn instantiate_contract<Args, C>(
        self,
        params: &CreateParams<T, Args, C>,
    ) -> Result<T::AccountId>
    where
        Args: scale::Encode,
    {
        env::instantiate_contract::<T, Args, C>(params)
    }

    /// Restores a smart contract in tombstone state.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::restore_contract`]
    pub fn restore_contract(
        self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) {
        env::restore_contract::<T>(account_id, code_hash, rent_allowance, filtered_keys)
    }

    /// Terminates the existence of a smart contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::terminate_contract`]
    pub fn terminate_contract(self, beneficiary: T::AccountId) -> !
    where
        T: EnvTypes,
    {
        env::terminate_contract::<T>(beneficiary)
    }

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::transfer`]
    pub fn transfer(self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: EnvTypes,
    {
        env::transfer::<T>(destination, value)
    }

    /// Returns a random hash seed.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::random`]
    pub fn random(self, subject: &[u8]) -> T::Hash
    where
        T: EnvTypes,
    {
        env::random::<T>(subject).expect("couldn't decode randomized hash")
    }
}
