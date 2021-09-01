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
    RentParams,
    RentStatus,
    Result,
    ReturnFlags,
    TypedEnvBackend,
};
use ink_engine::{
    ext,
    ext::Engine,
};
use ink_primitives::Key;

/// The capacity of the static buffer.
/// This is the same size as the ink! on-chain environment. We chose to use the same size
/// to be as close to the on-chain behavior as possible.
const BUFFER_SIZE: usize = 1 << 14; // 16 kB

impl CryptoHash for Blake2x128 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 16];
        static_assertions::assert_type_eq_all!(
            <Blake2x128 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 16);
        Engine::hash_blake2_128(input, output);
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
        Engine::hash_blake2_256(input, output);
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
        Engine::hash_sha2_256(input, output);
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
        Engine::hash_keccak_256(input, output);
    }
}

impl From<ext::Error> for crate::Error {
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

#[derive(Default)]
pub struct TopicsBuilder {
    pub topics: Vec<Vec<u8>>,
}

impl<E> TopicsBuilderBackend<E> for TopicsBuilder
where
    E: Environment,
{
    type Output = Vec<u8>;

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
    fn get_property<T>(
        &mut self,
        ext_fn: fn(engine: &Engine, output: &mut &mut [u8]),
    ) -> Result<T>
    where
        T: scale::Decode,
    {
        let mut full_scope: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let full_scope = &mut &mut full_scope[..];
        ext_fn(&self.engine, full_scope);
        scale::Decode::decode(&mut &full_scope[..]).map_err(Into::into)
    }
}

impl EnvBackend for EnvInstance {
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V)
    where
        V: scale::Encode,
    {
        let v = scale::Encode::encode(value);
        self.engine.set_storage(key.as_bytes(), &v[..]);
    }

    fn get_contract_storage<R>(&mut self, key: &Key) -> Result<Option<R>>
    where
        R: scale::Decode,
    {
        let mut output: [u8; 9600] = [0; 9600];
        match self
            .engine
            .get_storage(key.as_bytes(), &mut &mut output[..])
        {
            Ok(_) => (),
            Err(ext::Error::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = scale::Decode::decode(&mut &output[..])?;
        Ok(Some(decoded))
    }

    fn clear_contract_storage(&mut self, key: &Key) {
        self.engine.clear_storage(key.as_bytes())
    }

    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        unimplemented!("the experimental off chain env does not implement `seal_input`")
    }

    fn return_value<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: scale::Encode,
    {
        unimplemented!(
            "the experimental off chain env does not implement `seal_return_value`"
        )
    }

    fn debug_message(&mut self, message: &str) {
        self.engine.debug_message(message)
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

        status_to_result(self.engine.call_chain_extension(
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
        self.get_property::<T::AccountId>(Engine::caller)
    }

    fn transferred_balance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(Engine::value_transferred)
    }

    fn gas_left<T: Environment>(&mut self) -> Result<u64> {
        self.get_property::<u64>(Engine::gas_left)
    }

    fn block_timestamp<T: Environment>(&mut self) -> Result<T::Timestamp> {
        self.get_property::<T::Timestamp>(Engine::block_timestamp)
    }

    fn account_id<T: Environment>(&mut self) -> Result<T::AccountId> {
        self.get_property::<T::AccountId>(Engine::address)
    }

    fn balance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(Engine::balance)
    }

    fn rent_allowance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(Engine::rent_allowance)
    }

    fn rent_params<T>(&mut self) -> Result<RentParams<T>>
    where
        T: Environment,
    {
        unimplemented!("off-chain environment does not support rent params")
    }

    fn rent_status<T>(
        &mut self,
        _at_refcount: Option<core::num::NonZeroU32>,
    ) -> Result<RentStatus<T>>
    where
        T: Environment,
    {
        unimplemented!("off-chain environment does not support rent status")
    }

    fn block_number<T: Environment>(&mut self) -> Result<T::BlockNumber> {
        self.get_property::<T::BlockNumber>(Engine::block_number)
    }

    fn minimum_balance<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(Engine::minimum_balance)
    }

    fn tombstone_deposit<T: Environment>(&mut self) -> Result<T::Balance> {
        self.get_property::<T::Balance>(Engine::tombstone_deposit)
    }

    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: Environment,
        Event: Topics + scale::Encode,
    {
        let builder = TopicsBuilder::default();
        let enc_topics = event.topics::<T, _>(builder.into());
        let enc_data = &scale::Encode::encode(&event)[..];
        self.engine.deposit_event(&enc_topics[..], enc_data);
    }

    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: Environment,
    {
        let buffer = &scale::Encode::encode(&new_value)[..];
        self.engine.set_rent_allowance(buffer)
    }

    fn invoke_contract<T, Args>(
        &mut self,
        _call_params: &CallParams<T, Args, ()>,
    ) -> Result<()>
    where
        T: Environment,
        Args: scale::Encode,
    {
        unimplemented!("off-chain environment does not support contract invocation")
    }

    fn eval_contract<T, Args, R>(
        &mut self,
        _call_params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        T: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        unimplemented!("off-chain environment does not support contract evaluation")
    }

    fn instantiate_contract<T, Args, Salt, C>(
        &mut self,
        _params: &CreateParams<T, Args, Salt, C>,
    ) -> Result<T::AccountId>
    where
        T: Environment,
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        unimplemented!("off-chain environment does not support contract instantiation")
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

        let filtered: Vec<&[u8]> =
            filtered_keys.iter().map(|k| &k.as_bytes()[..]).collect();
        self.engine.restore_to(
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
        self.engine.terminate(&buffer[..])
    }

    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: Environment,
    {
        let enc_destination = &scale::Encode::encode(&destination)[..];
        let enc_value = &scale::Encode::encode(&value)[..];
        self.engine
            .transfer(enc_destination, enc_value)
            .map_err(Into::into)
    }

    fn weight_to_fee<T: Environment>(&mut self, gas: u64) -> Result<T::Balance> {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        self.engine.weight_to_fee(gas, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<(T::Hash, T::BlockNumber)>
    where
        T: Environment,
    {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        self.engine.random(subject, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }
}
