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

use crate::env::{
    call::{
        CallData,
        CallParams,
        InstantiateParams,
        ReturnType,
    },
    EnvTypes,
    Result,
    Topics,
};
use ink_primitives::Key;

/// Environmental contract functionality that does not require `EnvTypes`.
pub trait Env {
    /// Writes the value to the contract storage under the given key.
    fn set_contract_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode;

    /// Returns the value stored under the given key in the contract's storage if any.
    ///
    /// # Errors
    ///
    /// - If the decoding of the typed value failed
    fn get_contract_storage<R>(&mut self, key: Key) -> Option<Result<R>>
    where
        R: scale::Decode;

    /// Clears the contract's storage key entry.
    fn clear_contract_storage(&mut self, key: Key);

    /// Returns the value from the *runtime* storage at the position of the key if any.
    ///
    /// # Errors
    ///
    /// - If the decoding of the typed value failed
    fn get_runtime_storage<R>(&mut self, runtime_key: &[u8]) -> Option<Result<R>>
    where
        R: scale::Decode;

    /// Returns the input to the executed contract.
    ///
    /// # Note
    ///
    /// - The input is the 4-bytes selector followed by the arguments
    ///   of the called function in their SCALE encoded representation.
    /// - This property must be received as the first action an executed
    ///   contract to its environment and can only be queried once.
    ///   The environment access asserts this guarantee.
    fn input(&mut self) -> Result<CallData>;

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// The setting of this property must be the last interaction between
    /// the executed contract and its environment.
    /// The environment access asserts this guarantee.
    fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode;

    /// Prints the given contents to the console log.
    fn println(&mut self, content: &str);

    /// Conducts the SHA2 256-bit hash of the input
    /// puts the result into the output buffer.
    fn hash_sha2_256(input: &[u8], output: &mut [u8; 32]);

    /// Conducts the KECCAK 256-bit hash of the input
    /// puts the result into the output buffer.
    fn hash_keccak_256(input: &[u8], output: &mut [u8; 32]);

    /// Conducts the BLAKE2 256-bit hash of the input
    /// puts the result into the output buffer.
    fn hash_blake2_256(input: &[u8], output: &mut [u8; 32]);

    /// Conducts the BLAKE2 128-bit hash of the input
    /// puts the result into the output buffer.
    fn hash_blake2_128(input: &[u8], output: &mut [u8; 16]);
}

/// Environmental contract functionality.
pub trait TypedEnv: Env {
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::caller`]
    fn caller<T: EnvTypes>(&mut self) -> Result<T::AccountId>;

    /// Returns the transferred balance for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::transferred_balance`]
    fn transferred_balance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the current price for gas.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::gas_price`]
    fn gas_price<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::gas_left`]
    fn gas_left<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the timestamp of the current block.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::block_timestamp`]
    fn block_timestamp<T: EnvTypes>(&mut self) -> Result<T::Timestamp>;

    /// Returns the address of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::account_id`]
    fn account_id<T: EnvTypes>(&mut self) -> Result<T::AccountId>;

    /// Returns the balance of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::balance`]
    fn balance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the current rent allowance for the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::rent_allowance`]
    fn rent_allowance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the current block number.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::block_number`]
    fn block_number<T: EnvTypes>(&mut self) -> Result<T::BlockNumber>;

    /// Returns the minimum balance of the contracts chain.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::minimum_balance`]
    fn minimum_balance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the tombstone deposit of the contract chain.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::tombstone_deposit`]
    fn tombstone_deposit<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Emits an event with the given event data.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::emit_event`]
    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: EnvTypes,
        Event: Topics<T> + scale::Encode;

    /// Sets the rent allowance of the executed contract to the new value.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::set_rent_allowance`]
    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: EnvTypes;

    /// Invokes a call of the runtime.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::invoke_runtime`]
    fn invoke_runtime<T>(&mut self, call: &T::Call) -> Result<()>
    where
        T: EnvTypes;

    /// Invokes a contract message.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::invoke_contract`]
    fn invoke_contract<T>(&mut self, call_data: &CallParams<T, ()>) -> Result<()>
    where
        T: EnvTypes;

    /// Evaluates a contract message and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::eval_contract`]
    fn eval_contract<T, R>(
        &mut self,
        call_data: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        T: EnvTypes,
        R: scale::Decode;

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::instantiate_contract`]
    fn instantiate_contract<T, C>(
        &mut self,
        params: &InstantiateParams<T, C>,
    ) -> Result<T::AccountId>
    where
        T: EnvTypes;

    /// Restores a smart contract tombstone.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::restore_contract`]
    fn restore_contract<T>(
        &mut self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) where
        T: EnvTypes;

    /// Terminates a smart contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::terminate_contract`]
    fn terminate_contract<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: EnvTypes;

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::transfer`]
    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: EnvTypes;

    /// Returns a random hash seed.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_core::env::random`]
    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes;
}
