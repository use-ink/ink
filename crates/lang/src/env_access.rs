// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
use ink_env::{
    call::{
        utils::ReturnType,
        CallParams,
        CreateParams,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    Environment,
    Result,
};
use ink_primitives::Key;

use crate::ChainExtensionInstance;

/// The environment of the compiled ink! smart contract.
pub trait ContractEnv {
    /// The environment type.
    type Env: ::ink_env::Environment;
}

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
    T: Environment,
    <T as Environment>::ChainExtension: ChainExtensionInstance,
{
    /// Allows to call one of the available defined chain extension methods.
    pub fn extension(
        self,
    ) -> <<T as Environment>::ChainExtension as ChainExtensionInstance>::Instance {
        <<T as Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
    }
}

impl<'a, T> EnvAccess<'a, T>
where
    T: Environment,
{
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::caller`]
    pub fn caller(self) -> T::AccountId {
        ink_env::caller::<T>().expect("couldn't decode caller")
    }

    /// Returns the transferred balance for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::transferred_balance`]
    pub fn transferred_balance(self) -> T::Balance {
        ink_env::transferred_balance::<T>().expect("couldn't decode transferred balance")
    }

    /// Returns the price for the specified amount of gas.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::weight_to_fee`]
    pub fn weight_to_fee(self, gas: u64) -> T::Balance {
        ink_env::weight_to_fee::<T>(gas).expect("couldn't decode weight fee")
    }

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::gas_left`]
    pub fn gas_left(self) -> T::Balance {
        ink_env::gas_left::<T>().expect("couldn't decode gas left")
    }

    /// Returns the timestamp of the current block.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::block_timestamp`]
    pub fn block_timestamp(self) -> T::Timestamp {
        ink_env::block_timestamp::<T>().expect("couldn't decode block time stamp")
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::account_id`]
    pub fn account_id(self) -> T::AccountId {
        ink_env::account_id::<T>().expect("couldn't decode contract account ID")
    }

    /// Returns the balance of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::balance`]
    pub fn balance(self) -> T::Balance {
        ink_env::balance::<T>().expect("couldn't decode contract balance")
    }

    /// Returns the current rent allowance for the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::rent_allowance`]
    pub fn rent_allowance(self) -> T::Balance {
        ink_env::rent_allowance::<T>().expect("couldn't decode contract rent allowance")
    }

    /// Returns the current block number.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::block_number`]
    pub fn block_number(self) -> T::BlockNumber {
        ink_env::block_number::<T>().expect("couldn't decode block number")
    }

    /// Returns the minimum balance that is required for creating an account.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::minimum_balance`]
    pub fn minimum_balance(self) -> T::Balance {
        ink_env::minimum_balance::<T>().expect("couldn't decode minimum account balance")
    }

    /// Returns the tombstone deposit for the contracts chain.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::tombstone_deposit`]
    pub fn tombstone_deposit(self) -> T::Balance {
        ink_env::tombstone_deposit::<T>().expect("couldn't decode tombstone deposits")
    }

    /// Sets the rent allowance of the executed contract to the new value.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::set_rent_allowance`]
    pub fn set_rent_allowance(self, new_value: T::Balance) {
        ink_env::set_rent_allowance::<T>(new_value)
    }

    /// Invokes a contract message.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::invoke_contract`]
    pub fn invoke_contract<Args>(self, params: &CallParams<T, Args, ()>) -> Result<()>
    where
        Args: scale::Encode,
    {
        ink_env::invoke_contract::<T, Args>(params)
    }

    /// Evaluates a contract message and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::eval_contract`]
    pub fn eval_contract<Args, R>(
        self,
        params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        Args: scale::Encode,
        R: scale::Decode,
    {
        ink_env::eval_contract::<T, Args, R>(params)
    }

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::instantiate_contract`]
    pub fn instantiate_contract<Args, Salt, C>(
        self,
        params: &CreateParams<T, Args, Salt, C>,
    ) -> Result<T::AccountId>
    where
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        ink_env::instantiate_contract::<T, Args, Salt, C>(params)
    }

    /// Restores a smart contract in tombstone state.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::restore_contract`]
    pub fn restore_contract(
        self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) {
        ink_env::restore_contract::<T>(
            account_id,
            code_hash,
            rent_allowance,
            filtered_keys,
        )
    }

    /// Terminates the existence of a smart contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::terminate_contract`]
    pub fn terminate_contract(self, beneficiary: T::AccountId) -> ! {
        ink_env::terminate_contract::<T>(beneficiary)
    }

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::transfer`]
    pub fn transfer(self, destination: T::AccountId, value: T::Balance) -> Result<()> {
        ink_env::transfer::<T>(destination, value)
    }

    /// Returns a random hash seed.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::random`]
    pub fn random(self, subject: &[u8]) -> T::Hash {
        ink_env::random::<T>(subject).expect("couldn't decode randomized hash")
    }

    /// Computes the hash of the given bytes using the cryptographic hash `H`.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::hash_bytes`]
    pub fn hash_bytes<H>(self, input: &[u8]) -> <H as HashOutput>::Type
    where
        H: CryptoHash,
    {
        let mut output = <H as HashOutput>::Type::default();
        ink_env::hash_bytes::<H>(input, &mut output);
        output
    }

    /// Computes the hash of the given SCALE encoded value using the cryptographic hash `H`.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::hash_encoded`]
    pub fn hash_encoded<H, V>(self, value: &V) -> <H as HashOutput>::Type
    where
        H: CryptoHash,
        V: scale::Encode,
    {
        let mut output = <H as HashOutput>::Type::default();
        ink_env::hash_encoded::<H, V>(value, &mut output);
        output
    }
}
