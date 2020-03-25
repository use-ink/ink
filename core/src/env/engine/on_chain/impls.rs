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

use super::{
    ext,
    EnvInstance,
};
use crate::env::{
    call::{
        CallData,
        CallParams,
        InstantiateParams,
        ReturnType,
    },
    Env,
    EnvTypes,
    Result,
    Topics,
    TypedEnv,
};
use ink_primitives::Key;

impl EnvInstance {
    /// Empties the contract-side scratch buffer.
    ///
    /// # Note
    ///
    /// This is useful to perform before invoking a series of
    /// [`WasmEnv::append_encode_into_buffer`].
    fn reset_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Resizes the amount of used bytes of the internal buffer.
    fn resize_buffer(&mut self, new_len: usize) {
        self.buffer.resize(new_len);
    }

    /// Reads the current scratch buffer into the contract-side buffer.
    ///
    /// Returns the amount of bytes read.
    fn read_scratch_buffer(&mut self) -> usize {
        let req_len = ext::scratch_size();
        self.resize_buffer(req_len);
        ext::scratch_read(&mut self.buffer[0..req_len], 0);
        req_len
    }

    /// Reads from the scratch buffer and directly decodes into a value of `T`.
    ///
    /// # Errors
    ///
    /// If the decoding into a value of `T` failed.
    fn decode_scratch_buffer<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        let req_len = self.read_scratch_buffer();
        scale::Decode::decode(&mut &self.buffer[0..req_len]).map_err(Into::into)
    }

    /// Encodes the value into the contract-side scratch buffer.
    fn encode_into_buffer<T>(&mut self, value: T)
    where
        T: scale::Encode,
    {
        self.reset_buffer();
        scale::Encode::encode_to(&value, &mut self.buffer);
    }

    /// Appends the encoded value into the contract-side scratch buffer
    /// and returns the byte ranges into the encoded region.
    fn append_encode_into_buffer<T>(&mut self, value: T) -> core::ops::Range<usize>
    where
        T: scale::Encode,
    {
        let start = self.buffer.len();
        scale::Encode::encode_to(&value, &mut self.buffer);
        let end = self.buffer.len();
        core::ops::Range { start, end }
    }

    /// Returns the contract property value.
    fn get_property<T>(&mut self, ext_fn: fn()) -> Result<T>
    where
        T: scale::Decode,
    {
        ext_fn();
        self.decode_scratch_buffer().map_err(Into::into)
    }

    /// Reusable implementation for invoking another contract message.
    fn invoke_contract_impl<T, RetType>(
        &mut self,
        call_params: &CallParams<T, RetType>,
    ) -> Result<()>
    where
        T: EnvTypes,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `call_data`, `endowment` and `call_data`
        // in order and remember their encoded regions within the buffer.
        let callee = self.append_encode_into_buffer(call_params.callee());
        let transferred_value =
            self.append_encode_into_buffer(call_params.transferred_value());
        let call_data = self.append_encode_into_buffer(call_params.input_data());
        // Resolve the encoded regions into actual byte slices.
        let callee = &self.buffer[callee];
        let transferred_value = &self.buffer[transferred_value];
        let call_data = &self.buffer[call_data];
        // Perform the actual contract call.
        ext::call(
            callee,
            call_params.gas_limit(),
            transferred_value,
            call_data,
        )
    }
}

impl Env for EnvInstance {
    fn set_contract_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode,
    {
        self.encode_into_buffer(value);
        ext::set_storage(key.as_bytes(), &self.buffer[..]);
    }

    fn get_contract_storage<R>(&mut self, key: Key) -> Option<Result<R>>
    where
        R: scale::Decode,
    {
        if ext::get_storage(key.as_bytes()).is_err() {
            return None
        }
        Some(self.decode_scratch_buffer().map_err(Into::into))
    }

    fn clear_contract_storage(&mut self, key: Key) {
        ext::clear_storage(key.as_bytes())
    }

    fn get_runtime_storage<R>(&mut self, runtime_key: &[u8]) -> Option<Result<R>>
    where
        R: scale::Decode,
    {
        if ext::get_runtime_storage(runtime_key).is_err() {
            return None
        }
        Some(self.decode_scratch_buffer().map_err(Into::into))
    }

    fn input(&mut self) -> Result<CallData> {
        self.get_property::<CallData>(|| ())
    }

    fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode,
    {
        self.encode_into_buffer(return_value);
        ext::scratch_write(&self.buffer[..]);
    }

    fn println(&mut self, content: &str) {
        ext::println(content)
    }

    fn hash_keccak_256(input: &[u8], output: &mut [u8; 32]) {
        ext::hash_keccak_256(input, output)
    }

    fn hash_blake2_256(input: &[u8], output: &mut [u8; 32]) {
        ext::hash_blake2_256(input, output)
    }

    fn hash_blake2_128(input: &[u8], output: &mut [u8; 16]) {
        ext::hash_blake2_128(input, output)
    }

    fn hash_sha2_256(input: &[u8], output: &mut [u8; 32]) {
        ext::hash_sha2_256(input, output)
    }
}

impl TypedEnv for EnvInstance {
    fn caller<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        self.get_property::<T::AccountId>(ext::caller)
    }

    fn transferred_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::value_transferred)
    }

    fn gas_price<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::gas_price)
    }

    fn gas_left<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::gas_left)
    }

    fn block_timestamp<T: EnvTypes>(&mut self) -> Result<T::Timestamp> {
        self.get_property::<T::Timestamp>(ext::now)
    }

    fn account_id<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        self.get_property::<T::AccountId>(ext::address)
    }

    fn balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::balance)
    }

    fn rent_allowance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::rent_allowance)
    }

    fn block_number<T: EnvTypes>(&mut self) -> Result<T::BlockNumber> {
        self.get_property::<T::BlockNumber>(ext::block_number)
    }

    fn minimum_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::minimum_balance)
    }

    fn tombstone_deposit<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::tombstone_deposit)
    }

    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: EnvTypes,
        Event: Topics<T> + scale::Encode,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `topics` and the raw encoded `data`
        // in order and remember their encoded regions within the buffer.
        let topics = self.append_encode_into_buffer(event.topics());
        let data = self.append_encode_into_buffer(event);
        // Resolve the encoded regions into actual byte slices.
        let topics = &self.buffer[topics];
        let data = &self.buffer[data];
        // Do the actual depositing of the event.
        ext::deposit_event(topics, data);
    }

    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: EnvTypes,
    {
        self.encode_into_buffer(&new_value);
        ext::set_rent_allowance(&self.buffer[..])
    }

    fn invoke_runtime<T>(&mut self, call: &T::Call) -> Result<()>
    where
        T: EnvTypes,
    {
        self.encode_into_buffer(call);
        ext::dispatch_call(&self.buffer[..]);
        Ok(())
    }

    fn invoke_contract<T>(&mut self, call_params: &CallParams<T, ()>) -> Result<()>
    where
        T: EnvTypes,
    {
        self.invoke_contract_impl(call_params)
    }

    fn eval_contract<T, R>(
        &mut self,
        call_params: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        T: EnvTypes,
        R: scale::Decode,
    {
        self.invoke_contract_impl(call_params)?;
        self.decode_scratch_buffer().map_err(Into::into)
    }

    fn instantiate_contract<T, C>(
        &mut self,
        params: &InstantiateParams<T, C>,
    ) -> Result<T::AccountId>
    where
        T: EnvTypes,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `code_hash`, `endowment` and `create_data`
        // in order and remember their encoded regions within the buffer.
        let code_hash = self.append_encode_into_buffer(params.code_hash());
        let endowment = self.append_encode_into_buffer(params.endowment());
        let create_data = self.append_encode_into_buffer(params.input_data());
        // Resolve the encoded regions into actual byte slices.
        let code_hash = &self.buffer[code_hash];
        let endowment = &self.buffer[endowment];
        let create_data = &self.buffer[create_data];
        // Do the actual contract instantiation.
        ext::create(code_hash, params.gas_limit(), endowment, create_data)?;
        // At this point our contract instantiation was successful
        // and we can now fetch the returned data and decode it for
        // the result value.
        self.decode_scratch_buffer().map_err(Into::into)
    }

    fn restore_contract<T>(
        &mut self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) where
        T: EnvTypes,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `account_id`, `code_hash` and `rent_allowance`
        // and `filtered_keys` in order and remember their encoded regions
        // within the buffer.
        let account_id = self.append_encode_into_buffer(account_id);
        let code_hash = self.append_encode_into_buffer(code_hash);
        let rent_allowance = self.append_encode_into_buffer(rent_allowance);
        // Resolve the encoded regions into actual byte slices.
        let account_id = &self.buffer[account_id];
        let code_hash = &self.buffer[code_hash];
        let rent_allowance = &self.buffer[rent_allowance];
        // Perform the actual contract restoration.
        ext::restore_to(account_id, code_hash, rent_allowance, filtered_keys);
    }

    fn terminate_contract<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: EnvTypes,
    {
        self.encode_into_buffer(beneficiary);
        ext::terminate(&self.buffer[..]);
    }

    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: EnvTypes,
    {
        // Reset the contract-side buffer to append onto clean slate.
        self.reset_buffer();
        // Append the encoded `destination` and `value` in order and remember
        // their encoded regions within the buffer.
        let destination = self.append_encode_into_buffer(destination);
        let value = self.append_encode_into_buffer(value);
        // Resolve the encoded regions into actual byte slices.
        let destination = &self.buffer[destination];
        let value = &self.buffer[value];
        // Perform the actual transfer call.
        ext::transfer(destination, value)
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        ext::random_seed(subject);
        self.decode_scratch_buffer().map_err(Into::into)
    }
}
