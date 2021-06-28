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

use super::{
    ext,
    EnvInstance,
    Error as ExtError,
    ScopedBuffer,
};
use crate::{
    call::{
        utils::ReturnType,
        CallParams,
        CreateParams,
    },
    hash::{
        Blake2x128,
        Blake2x256,
        CryptoHash,
        HashOutput,
        Keccak256,
        Sha2x256,
    },
    topics::{
        Topics,
        TopicsBuilderBackend,
    },
    types::{
        RentParams,
        RentStatus,
    },
    Clear,
    EnvBackend,
    Environment,
    Error,
    Result,
    ReturnFlags,
    TypedEnvBackend,
};
use ink_primitives::Key;

impl CryptoHash for Blake2x128 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 16];
        static_assertions::assert_type_eq_all!(
            <Blake2x128 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 16);
        ext::hash_blake2_128(input, output);
    }
}

impl CryptoHash for Blake2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Blake2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 32);
        ext::hash_blake2_256(input, output);
    }
}

impl CryptoHash for Sha2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Sha2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 32);
        ext::hash_sha2_256(input, output);
    }
}

impl CryptoHash for Keccak256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Keccak256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 32);
        ext::hash_keccak_256(input, output);
    }
}

impl From<ext::Error> for Error {
    fn from(ext_error: ext::Error) -> Self {
        match ext_error {
            ext::Error::Unknown => Self::Unknown,
            ext::Error::CalleeTrapped => Self::CalleeTrapped,
            ext::Error::CalleeReverted => Self::CalleeReverted,
            ext::Error::KeyNotFound => Self::KeyNotFound,
            ext::Error::BelowSubsistenceThreshold => Self::BelowSubsistenceThreshold,
            ext::Error::TransferFailed => Self::TransferFailed,
            ext::Error::NewContractNotFunded => Self::NewContractNotFunded,
            ext::Error::CodeNotFound => Self::CodeNotFound,
            ext::Error::NotCallable => Self::NotCallable,
            ext::Error::LoggingDisabled => Self::LoggingDisabled,
        }
    }
}

pub struct TopicsBuilder<'a, E> {
    scoped_buffer: ScopedBuffer<'a>,
    marker: core::marker::PhantomData<fn() -> E>,
}

impl<'a, E> From<ScopedBuffer<'a>> for TopicsBuilder<'a, E>
where
    E: Environment,
{
    fn from(scoped_buffer: ScopedBuffer<'a>) -> Self {
        Self {
            scoped_buffer,
            marker: Default::default(),
        }
    }
}

impl<'a, E> TopicsBuilderBackend<E> for TopicsBuilder<'a, E>
where
    E: Environment,
{
    type Output = (ScopedBuffer<'a>, &'a mut [u8]);

    fn expect(&mut self, expected_topics: usize) {
        self.scoped_buffer
            .append_encoded(&scale::Compact(expected_topics as u32));
    }

    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: scale::Encode,
    {
        let mut split = self.scoped_buffer.split();
        let encoded = split.take_encoded(topic_value);
        let len_encoded = encoded.len();
        let mut result = <E as Environment>::Hash::clear();
        let len_result = result.as_ref().len();
        if len_encoded <= len_result {
            result.as_mut()[..len_encoded].copy_from_slice(encoded);
        } else {
            let mut hash_output = <Blake2x256 as HashOutput>::Type::default();
            <Blake2x256 as CryptoHash>::hash(encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
        }
        self.scoped_buffer.append_encoded(&result);
    }

    fn output(mut self) -> Self::Output {
        let encoded_topics = self.scoped_buffer.take_appended();
        (self.scoped_buffer, encoded_topics)
    }
}

impl EnvInstance {
    /// Returns a new scoped buffer for the entire scope of the static 16 kB buffer.
    fn scoped_buffer(&mut self) -> ScopedBuffer {
        ScopedBuffer::from(&mut self.buffer[..])
    }

    /// Returns the contract property value.
    fn get_property<T>(&mut self, ext_fn: fn(output: &mut &mut [u8])) -> Result<T>
    where
        T: scale::Decode,
    {
        let full_scope = &mut self.scoped_buffer().take_rest();
        ext_fn(full_scope);
        scale::Decode::decode(&mut &full_scope[..]).map_err(Into::into)
    }

    /// Reusable implementation for invoking another contract message.
    fn invoke_contract_impl<T, Args, RetType, R>(
        &mut self,
        params: &CallParams<T, Args, RetType>,
    ) -> Result<R>
    where
        T: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        let mut scope = self.scoped_buffer();
        let gas_limit = params.gas_limit();
        let enc_callee = scope.take_encoded(params.callee());
        let enc_transferred_value = scope.take_encoded(params.transferred_value());
        let enc_input = scope.take_encoded(params.exec_input());
        let output = &mut scope.take_rest();
        ext::call(
            enc_callee,
            gas_limit,
            enc_transferred_value,
            enc_input,
            output,
        )?;
        let decoded = scale::Decode::decode(&mut &output[..])?;
        Ok(decoded)
    }
}

impl EnvBackend for EnvInstance {
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V)
    where
        V: scale::Encode,
    {
        let buffer = self.scoped_buffer().take_encoded(value);
        ext::set_storage(key.as_bytes(), &buffer[..]);
    }

    fn get_contract_storage<R>(&mut self, key: &Key) -> Result<Option<R>>
    where
        R: scale::Decode,
    {
        let output = &mut self.scoped_buffer().take_rest();
        match ext::get_storage(key.as_bytes(), output) {
            Ok(_) => (),
            Err(ExtError::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = scale::Decode::decode(&mut &output[..])?;
        Ok(Some(decoded))
    }

    fn clear_contract_storage(&mut self, key: &Key) {
        ext::clear_storage(key.as_bytes())
    }

    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        self.get_property::<T>(ext::input)
    }

    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: scale::Encode,
    {
        let mut scope = self.scoped_buffer();
        let enc_return_value = scope.take_encoded(return_value);
        ext::return_value(flags, enc_return_value);
    }

    fn debug_message(&mut self, content: &str) {
        ext::debug_message(content)
    }

    fn hash_bytes<H>(&mut self, input: &[u8], output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
    {
        <H as CryptoHash>::hash(input, output)
    }

    fn hash_encoded<H, T>(&mut self, input: &T, output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
        T: scale::Encode,
    {
        let mut scope = self.scoped_buffer();
        let enc_input = scope.take_encoded(input);
        <H as CryptoHash>::hash(enc_input, output)
    }

    fn call_chain_extension<I, T, E, ErrorCode, F, D>(
        &mut self,
        func_id: u32,
        input: &I,
        status_to_result: F,
        decode_to_result: D,
    ) -> ::core::result::Result<T, E>
    where
        I: scale::Encode,
        T: scale::Decode,
        E: From<ErrorCode>,
        F: FnOnce(u32) -> ::core::result::Result<(), ErrorCode>,
        D: FnOnce(&[u8]) -> ::core::result::Result<T, E>,
    {
        let mut scope = self.scoped_buffer();
        let enc_input = scope.take_encoded(input);
        let output = &mut scope.take_rest();
        status_to_result(ext::call_chain_extension(func_id, enc_input, output))?;
        let decoded = decode_to_result(&mut &output[..])?;
        Ok(decoded)
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller<T: Environment>(&mut self) -> Result<T::AccountId> {
        self.get_property::<T::AccountId>(ext::caller)
    }

    fn transferred_balance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::value_transferred)
    }

    fn gas_left<T: Environment>(&mut self) -> Result<u64> {
        self.get_property::<u64>(ext::gas_left)
    }

    fn block_timestamp<T: Environment>(&mut self) -> Result<T::Timestamp> {
        self.get_property::<T::Timestamp>(ext::now)
    }

    fn account_id<T: Environment>(&mut self) -> Result<T::AccountId> {
        self.get_property::<T::AccountId>(ext::address)
    }

    fn balance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::balance)
    }

    fn rent_allowance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::rent_allowance)
    }

    fn block_number<T: Environment>(&mut self) -> Result<T::BlockNumber> {
        self.get_property::<T::BlockNumber>(ext::block_number)
    }

    fn minimum_balance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::minimum_balance)
    }

    fn tombstone_deposit<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::tombstone_deposit)
    }

    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: Environment,
        Event: Topics + scale::Encode,
    {
        let (mut scope, enc_topics) =
            event.topics::<T, _>(TopicsBuilder::from(self.scoped_buffer()).into());
        let enc_data = scope.take_encoded(&event);
        ext::deposit_event(enc_topics, enc_data);
    }

    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: Environment,
    {
        let buffer = self.scoped_buffer().take_encoded(&new_value);
        ext::set_rent_allowance(&buffer[..])
    }

    fn rent_params<T>(&mut self) -> Result<RentParams<T>>
    where
        T: Environment,
    {
        self.get_property::<RentParams<T>>(ext::rent_params)
    }

    fn rent_status<T>(
        &mut self,
        at_refcount: Option<core::num::NonZeroU32>,
    ) -> Result<RentStatus<T>>
    where
        T: Environment,
    {
        let output = &mut self.scoped_buffer().take_rest();
        ext::rent_status(at_refcount, output);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }

    fn invoke_contract<T, Args>(
        &mut self,
        call_params: &CallParams<T, Args, ()>,
    ) -> Result<()>
    where
        T: Environment,
        Args: scale::Encode,
    {
        self.invoke_contract_impl(call_params)
    }

    fn eval_contract<T, Args, R>(
        &mut self,
        call_params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        T: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        self.invoke_contract_impl(call_params)
    }

    fn instantiate_contract<T, Args, Salt, C>(
        &mut self,
        params: &CreateParams<T, Args, Salt, C>,
    ) -> Result<T::AccountId>
    where
        T: Environment,
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        let mut scoped = self.scoped_buffer();
        let gas_limit = params.gas_limit();
        let enc_code_hash = scoped.take_encoded(params.code_hash());
        let enc_endowment = scoped.take_encoded(params.endowment());
        let enc_input = scoped.take_encoded(params.exec_input());
        // We support `AccountId` types with an encoding that requires up to
        // 1024 bytes. Beyond that limit ink! contracts will trap for now.
        // In the default configuration encoded `AccountId` require 32 bytes.
        let out_address = &mut scoped.take(1024);
        let salt = params.salt_bytes().as_ref();
        let out_return_value = &mut scoped.take_rest();
        // We currently do nothing with the `out_return_value` buffer.
        // This should change in the future but for that we need to add support
        // for constructors that may return values.
        // This is useful to support fallible constructors for example.
        ext::instantiate(
            enc_code_hash,
            gas_limit,
            enc_endowment,
            enc_input,
            out_address,
            out_return_value,
            salt,
        )?;
        let account_id = scale::Decode::decode(&mut &out_address[..])?;
        Ok(account_id)
    }

    fn restore_contract<T>(
        &mut self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) where
        T: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_account_id = scope.take_encoded(&account_id);
        let enc_code_hash = scope.take_encoded(&code_hash);
        let enc_rent_allowance = scope.take_encoded(&rent_allowance);
        ext::restore_to(
            enc_account_id,
            enc_code_hash,
            enc_rent_allowance,
            filtered_keys,
        );
    }

    fn terminate_contract<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: Environment,
    {
        let buffer = self.scoped_buffer().take_encoded(&beneficiary);
        ext::terminate(&buffer[..]);
    }

    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_destination = scope.take_encoded(&destination);
        let enc_value = scope.take_encoded(&value);
        ext::transfer(enc_destination, enc_value).map_err(Into::into)
    }

    fn weight_to_fee<T: Environment>(&mut self, gas: u64) -> Result<T::Balance> {
        let output = &mut self.scoped_buffer().take_rest();
        ext::weight_to_fee(gas, output);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<(T::Hash, T::BlockNumber)>
    where
        T: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_subject = scope.take_bytes(subject);
        let output = &mut scope.take_rest();
        ext::random(enc_subject, output);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }
}
