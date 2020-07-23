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
        CallParams,
        InstantiateParams,
        ReturnType,
    },
    Env,
    EnvError,
    EnvTypes,
    Result,
    ReturnFlags,
    Topics,
    TypedEnv,
};
use ink_primitives::Key;

impl From<ext::Error> for EnvError {
    fn from(ext_error: ext::Error) -> Self {
        match ext_error {
            ext::Error::UnknownError => Self::UnknownError,
            ext::Error::CalleeTrapped => Self::ContractCallTrapped,
            ext::Error::CalleeReverted => Self::ContractCallReverted,
            ext::Error::KeyNotFound => Self::MissingContractStorageEntry,
        }
    }
}

pub struct EncodeScope<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> From<&'a mut [u8]> for EncodeScope<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer, len: 0 }
    }
}

impl<'a> EncodeScope<'a> {
    /// Returns the capacity of the encoded scope.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the length of the encoded scope.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the internal mutable byte slice.
    pub fn into_buffer(self) -> &'a mut [u8] {
        self.buffer
    }
}

impl<'a> scale::Output for EncodeScope<'a> {
    fn write(&mut self, bytes: &[u8]) {
        assert!(self.len() + bytes.len() <= self.capacity());
        let start = self.len;
        let len_bytes = bytes.len();
        self.buffer[start..(start + len_bytes)].copy_from_slice(bytes);
        self.len += len_bytes;
    }

    fn push_byte(&mut self, byte: u8) {
        assert_ne!(self.len(), self.capacity());
        self.buffer[self.len] = byte;
        self.len += 1;
    }
}

/// Scoped access to an underlying bytes buffer.
///
/// # Note
///
/// This is used to efficiently chunk up ink!'s internal static 16kB buffer
/// into smaller sub buffers for processing different parts of computations.
#[derive(Debug)]
pub struct ScopedBuffer<'a> {
    buffer: &'a mut [u8],
}

impl<'a> From<&'a mut [u8]> for ScopedBuffer<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer }
    }
}

impl<'a> ScopedBuffer<'a> {
    /// Returns the first `len` bytes of the buffer as mutable slice.
    pub fn take(&mut self, len: usize) -> &'a mut [u8] {
        assert!(len <= self.buffer.len());
        let len_before = self.buffer.len();
        let buffer = core::mem::take(&mut self.buffer);
        let (lhs, rhs) = buffer.split_at_mut(len);
        self.buffer = rhs;
        debug_assert_eq!(lhs.len(), len);
        let len_after = self.buffer.len();
        debug_assert_eq!(len_before - len_after, len);
        lhs
    }

    /// Encode the given value into the scoped buffer and return the sub slice
    /// containing all the encoded bytes.
    pub fn take_encoded<T>(&mut self, value: &T) -> &'a mut [u8]
    where
        T: scale::Encode,
    {
        let buffer = core::mem::take(&mut self.buffer);
        let mut encode_scope = EncodeScope::from(buffer);
        scale::Encode::encode_to(&value, &mut encode_scope);
        let encode_len = encode_scope.len();
        core::mem::replace(&mut self.buffer, encode_scope.into_buffer());
        self.take(encode_len)
    }

    /// Returns all of the remaining bytes of the buffer as mutable slice.
    pub fn take_rest(self) -> &'a mut [u8] {
        assert!(!self.buffer.is_empty());
        self.buffer
    }
}

impl EnvInstance {
    /// Returns a new scoped buffer for the entire scope of the static 16kB buffer.
    fn scoped_buffer(&mut self) -> ScopedBuffer {
        ScopedBuffer::from(&mut self.buffer[..])
    }

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
        todo!()
        // let req_len = ext::scratch_size();
        // self.resize_buffer(req_len);
        // ext::scratch_read(&mut self.buffer[0..req_len], 0);
        // req_len
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
        todo!()
        // let req_len = self.read_scratch_buffer();
        // scale::Decode::decode(&mut &self.buffer[0..req_len]).map_err(Into::into)
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
    fn invoke_contract_impl<T, Args, RetType>(
        &mut self,
        call_params: &CallParams<T, Args, RetType>,
    ) -> Result<()>
    where
        T: EnvTypes,
        Args: scale::Encode,
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
        // // Perform the actual contract call.
        // ext::call(
        //     callee,
        //     call_params.gas_limit(),
        //     transferred_value,
        //     call_data,
        // )
        todo!()
    }
}

impl Env for EnvInstance {
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V)
    where
        V: scale::Encode,
    {
        self.encode_into_buffer(value);
        ext::set_storage(key.as_bytes(), &self.buffer[..]);
    }

    fn get_contract_storage<R>(&mut self, key: &Key) -> Option<Result<R>>
    where
        R: scale::Decode,
    {
        todo!()
        // if ext::get_storage(key.as_bytes()).is_err() {
        //     return None
        // }
        // Some(self.decode_scratch_buffer().map_err(Into::into))
    }

    fn clear_contract_storage(&mut self, key: &Key) {
        ext::clear_storage(key.as_bytes())
    }

    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        // self.get_property::<T>(|| ())
        todo!()
    }

    fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode,
    {
        todo!()
        // self.encode_into_buffer(return_value);
        // ext::scratch_write(&self.buffer[..]);
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
        // self.get_property::<T::AccountId>(ext::caller)
        todo!()
    }

    fn transferred_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        // self.get_property::<T::Balance>(ext::value_transferred)
        todo!()
    }

    fn gas_left<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        // self.get_property::<T::Balance>(ext::gas_left)
        todo!()
    }

    fn block_timestamp<T: EnvTypes>(&mut self) -> Result<T::Timestamp> {
        // self.get_property::<T::Timestamp>(ext::now)
        todo!()
    }

    fn account_id<T: EnvTypes>(&mut self) -> Result<T::AccountId> {
        // self.get_property::<T::AccountId>(ext::address)
        todo!()
    }

    fn balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        // self.get_property::<T::Balance>(ext::balance)
        todo!()
    }

    fn rent_allowance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        // self.get_property::<T::Balance>(ext::rent_allowance)
        todo!()
    }

    fn block_number<T: EnvTypes>(&mut self) -> Result<T::BlockNumber> {
        // self.get_property::<T::BlockNumber>(ext::block_number)
        todo!()
    }

    fn minimum_balance<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        // self.get_property::<T::Balance>(ext::minimum_balance)
        todo!()
    }

    fn tombstone_deposit<T: EnvTypes>(&mut self) -> Result<T::Balance> {
        // self.get_property::<T::Balance>(ext::tombstone_deposit)
        todo!()
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

    fn invoke_contract<T, Args>(
        &mut self,
        call_params: &CallParams<T, Args, ()>,
    ) -> Result<()>
    where
        T: EnvTypes,
        Args: scale::Encode,
    {
        // self.invoke_contract_impl(call_params)
        todo!()
    }

    fn eval_contract<T, Args, R>(
        &mut self,
        call_params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        T: EnvTypes,
        Args: scale::Encode,
        R: scale::Decode,
    {
        // self.invoke_contract_impl(call_params)?;
        // self.decode_scratch_buffer().map_err(Into::into)
        todo!()
    }

    fn instantiate_contract<T, Args, C>(
        &mut self,
        params: &InstantiateParams<T, Args, C>,
    ) -> Result<T::AccountId>
    where
        T: EnvTypes,
        Args: scale::Encode,
    {
        todo!()
        // // Reset the contract-side buffer to append onto clean slate.
        // self.reset_buffer();
        // // Append the encoded `code_hash`, `endowment` and `create_data`
        // // in order and remember their encoded regions within the buffer.
        // let code_hash = self.append_encode_into_buffer(params.code_hash());
        // let endowment = self.append_encode_into_buffer(params.endowment());
        // let create_data = self.append_encode_into_buffer(params.input_data());
        // // Resolve the encoded regions into actual byte slices.
        // let code_hash = &self.buffer[code_hash];
        // let endowment = &self.buffer[endowment];
        // let create_data = &self.buffer[create_data];
        // // Do the actual contract instantiation.
        // ext::create(code_hash, params.gas_limit(), endowment, create_data)?;
        // // At this point our contract instantiation was successful
        // // and we can now fetch the returned data and decode it for
        // // the result value.
        // self.decode_scratch_buffer().map_err(Into::into)
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
        todo!()
        // // Reset the contract-side buffer to append onto clean slate.
        // self.reset_buffer();
        // // Append the encoded `destination` and `value` in order and remember
        // // their encoded regions within the buffer.
        // let destination = self.append_encode_into_buffer(destination);
        // let value = self.append_encode_into_buffer(value);
        // // Resolve the encoded regions into actual byte slices.
        // let destination = &self.buffer[destination];
        // let value = &self.buffer[value];
        // // Perform the actual transfer call.
        // ext::transfer(destination, value)
    }

    fn gas_price<T: EnvTypes>(&mut self, gas: u64) -> Result<T::Balance> {
        todo!()
        // ext::gas_price(gas);
        // self.decode_scratch_buffer().map_err(Into::into)
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        todo!()
        // ext::random_seed(subject);
        // self.decode_scratch_buffer().map_err(Into::into)
    }
}
