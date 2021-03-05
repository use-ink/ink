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

use super::EnvInstance;
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
    Clear,
    EnvBackend,
    Environment,
    Result,
    ReturnFlags,
    TypedEnvBackend,
};
use ink_engine::ext;
use ink_primitives::Key;
// use test_api::EmittedEvent;

const BUFFER_SIZE: usize = 1024;

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

impl From<ext::Error> for crate::Error {
    fn from(ext_error: ext::Error) -> Self {
        match ext_error {
            ext::Error::UnknownError => Self::UnknownError,
            ext::Error::CalleeTrapped => Self::CalleeTrapped,
            ext::Error::CalleeReverted => Self::CalleeReverted,
            ext::Error::KeyNotFound => Self::KeyNotFound,
            ext::Error::BelowSubsistenceThreshold => Self::BelowSubsistenceThreshold,
            ext::Error::TransferFailed => Self::TransferFailed,
            ext::Error::NewContractNotFunded => Self::NewContractNotFunded,
            ext::Error::CodeNotFound => Self::CodeNotFound,
            ext::Error::NotCallable => Self::NotCallable,
        }
    }
}

#[derive(Default)]
pub struct TopicsBuilder {
    pub topics: Vec<Vec<u8>>,
}

impl<E> TopicsBuilderBackend<E> for TopicsBuilder
where
    E: Environment,
{
    type Output = Vec<u8>;
    // type Output = Vec<Vec<u8>>;

    fn expect(&mut self, _expected_topics: usize) {}

    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: scale::Encode,
    {
        let encoded = topic_value.encode();
        let len_encoded = encoded.len();
        let mut result = <E as Environment>::Hash::clear();
        let len_result = result.as_ref().len();
        if len_encoded <= len_result {
            result.as_mut()[..len_encoded].copy_from_slice(&encoded[..]);
        } else {
            let mut hash_output = <Blake2x256 as HashOutput>::Type::default();
            <Blake2x256 as CryptoHash>::hash(&encoded[..], &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
        }
        let off_hash = result.as_ref();
        let off_hash = off_hash.to_vec();
        debug_assert!(
            !self.topics.contains(&off_hash),
            "duplicate topic hash discovered!"
        );
        self.topics.push(off_hash);
    }

    fn output(self) -> Self::Output {
        let mut all: Vec<u8> = Vec::new();

        let topics_len_compact = &scale::Compact(self.topics.len() as u32);
        let topics_encoded = &scale::Encode::encode(&topics_len_compact)[..];
        all.append(&mut topics_encoded.to_vec());

        self.topics.into_iter().for_each(|mut v| all.append(&mut v));
        all
    }
}

impl EnvInstance {
    /// Returns the contract property value.
    fn get_property<T>(&mut self, ext_fn: fn(output: &mut &mut [u8])) -> Result<T>
    where
        T: scale::Decode,
    {
        let mut full_scope: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let full_scope = &mut &mut full_scope[..];
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
        let gas_limit = params.gas_limit();
        let enc_callee = &scale::Encode::encode(&params.callee())[..];
        let enc_transferred_value =
            &scale::Encode::encode(&params.transferred_value())[..];
        let enc_input = &scale::Encode::encode(&params.exec_input())[..];
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        ext::call(
            enc_callee,
            gas_limit,
            enc_transferred_value,
            enc_input,
            &mut &mut output[..],
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
        let v = scale::Encode::encode(value);
        ext::set_storage(key.as_bytes(), &v[..]);
    }

    fn get_contract_storage<R>(&mut self, key: &Key) -> Result<Option<R>>
    where
        R: scale::Decode,
    {
        let mut output: [u8; 9600] = [0; 9600];
        match ext::get_storage(key.as_bytes(), &mut &mut output[..]) {
            Ok(_) => (),
            Err(ext::Error::KeyNotFound) => return Ok(None),
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
        let enc_return_value = &scale::Encode::encode(return_value)[..];
        ext::return_value(flags.into_u32(), enc_return_value);
    }

    fn println(&mut self, content: &str) {
        ext::println(content)
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
        let enc_input = &scale::Encode::encode(input)[..];
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
        let enc_input = &scale::Encode::encode(input)[..];
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        status_to_result(ext::call_chain_extension(
            func_id,
            enc_input,
            &mut &mut output[..],
        ))?;
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

    fn gas_left<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(ext::gas_left)
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
        let builder = TopicsBuilder::default();
        let enc_topics = event.topics::<T, _>(builder.into());
        let enc_data = &scale::Encode::encode(&event)[..];
        ext::deposit_event(&enc_topics[..], enc_data);
    }

    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: Environment,
    {
        let buffer = &scale::Encode::encode(&new_value)[..];
        ext::set_rent_allowance(buffer)
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
        let gas_limit = params.gas_limit();
        let enc_code_hash = &scale::Encode::encode(&params.code_hash())[..];
        let enc_endowment = &scale::Encode::encode(&params.endowment())[..];
        let enc_input = &scale::Encode::encode(&params.exec_input())[..];

        // We support `AccountId` types with an encoding that requires up to
        // BUFFER_SIZE bytes. Beyond that limit ink! contracts will trap for now.
        // In the default configuration encoded `AccountId` require 32 bytes.
        let mut out_address: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let salt = params.salt_bytes().as_ref();
        let mut out_return_value: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        // We currently do nothing with the `out_return_value` buffer.
        // This should change in the future but for that we need to add support
        // for constructors that may return values.
        // This is useful to support fallible constructors for example.
        ext::instantiate(
            enc_code_hash,
            gas_limit,
            enc_endowment,
            enc_input,
            &mut &mut out_address[..],
            &mut &mut out_return_value[..],
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
        let enc_account_id = &scale::Encode::encode(&account_id)[..];
        let enc_code_hash = &scale::Encode::encode(&code_hash)[..];
        let enc_rent_allowance = &scale::Encode::encode(&rent_allowance)[..];

        let filtered: Vec<Vec<u8>> = filtered_keys
            .iter()
            .map(|k| k.as_bytes().to_vec())
            .collect();
        ext::restore_to(
            enc_account_id,
            enc_code_hash,
            enc_rent_allowance,
            &filtered[..],
        );
    }

    fn terminate_contract<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: Environment,
    {
        let buffer = scale::Encode::encode(&beneficiary);
        ext::terminate(&buffer[..]);
    }

    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: Environment,
    {
        let enc_destination = &scale::Encode::encode(&destination)[..];
        let enc_value = &scale::Encode::encode(&value)[..];
        ext::transfer(enc_destination, enc_value).map_err(Into::into)
    }

    fn weight_to_fee<T: Environment>(&mut self, gas: u64) -> Result<T::Balance> {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        ext::weight_to_fee(gas, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<T::Hash>
    where
        T: Environment,
    {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        ext::random(subject, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }
}
