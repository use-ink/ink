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
}

/// Environmental contract functionality.
pub trait TypedEnv: Env {
    /// Returns the address of the caller of the executed contract.
    fn caller<T: EnvTypes>(&mut self) -> Result<T::AccountId>;

    /// Returns the transferred balance for the contract execution.
    fn transferred_balance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the current price for gas.
    fn gas_price<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the amount of gas left for the contract execution.
    fn gas_left<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the timestamp of the current block.
    fn block_timestamp<T: EnvTypes>(&mut self) -> Result<T::Timestamp>;

    /// Returns the address of the executed contract.
    fn address<T: EnvTypes>(&mut self) -> Result<T::AccountId>;

    /// Returns the balance of the executed contract.
    fn balance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the current rent allowance for the executed contract.
    fn rent_allowance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the current block number.
    fn block_number<T: EnvTypes>(&mut self) -> Result<T::BlockNumber>;

    /// Returns the minimum balance of the contracts chain.
    fn minimum_balance<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Returns the tombstone deposit of the contract chain.
    fn tombstone_deposit<T: EnvTypes>(&mut self) -> Result<T::Balance>;

    /// Emits an event with the given event data.
    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: EnvTypes,
        Event: Topics<T> + scale::Encode;

    /// Sets the rent allowance of the executed contract to the new value.
    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: EnvTypes;

    /// Invokes a call of the runtime.
    fn invoke_runtime<T>(&mut self, call: &T::Call) -> Result<()>
    where
        T: EnvTypes;

    /// Invokes a contract message.
    ///
    /// # Errors
    ///
    /// If the called contract has trapped.
    fn invoke_contract<T>(&mut self, call_data: &CallParams<T, ()>) -> Result<()>
    where
        T: EnvTypes;

    /// Evaluates a contract message and returns its result.
    ///
    /// # Errors
    ///
    /// - If the called contract traps.
    /// - If the account ID is invalid.
    /// - If given too few endowment.
    /// - If arguments passed to the called contract are invalid.
    /// - If the called contract runs out of gas.
    fn eval_contract<T, R>(
        &mut self,
        call_data: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        T: EnvTypes,
        R: scale::Decode;

    /// Instantiates another contract.
    ///
    /// # Errors
    ///
    /// - If the instantiation process traps.
    /// - If the code hash is invalid.
    /// - If given too few endowment.
    /// - If the instantiation process runs out of gas.
    fn create_contract<T, C>(
        &mut self,
        params: &InstantiateParams<T, C>,
    ) -> Result<T::AccountId>
    where
        T: EnvTypes;

    /// Restores a smart contract tombstone.
    ///
    /// # Params
    ///
    /// - `account_id`: Encoded bytes of the `AccountId` of the to-be-restored contract.
    /// - `code_hash`: Encoded code hash of the to-be-restored contract.
    /// - `rent_allowance`: The encoded rent allowance of the restored contract
    ///                     upon successful restoration.
    /// - `filtered_keys`: Storage keys that will be ignored for the tombstone hash
    ///                    match calculation that decide whether the original contract
    ///                    storage and the storage of the restorer contract is equal.
    ///
    /// # Usage
    ///
    /// A smart contract that has too few funds to pay for its storage fees
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
    fn restore_contract<T>(
        &mut self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) where
        T: EnvTypes;

    /// Returns a random hash.
    ///
    /// # Note
    ///
    /// The subject buffer can be used to further randomize the hash.
    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes;
}
