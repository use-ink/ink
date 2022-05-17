// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
        Call,
        CallParams,
        CreateParams,
        DelegateCall,
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
    Error,
    Result,
    ReturnFlags,
    TypedEnvBackend,
};
use ink_engine::{
    ext,
    ext::Engine,
};
use ink_primitives::{
    Key,
    StorageKey,
    StorageKeyComposer,
};

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
            ext::Error::_BelowSubsistenceThreshold => Self::_BelowSubsistenceThreshold,
            ext::Error::TransferFailed => Self::TransferFailed,
            ext::Error::_EndowmentTooLow => Self::_EndowmentTooLow,
            ext::Error::CodeNotFound => Self::CodeNotFound,
            ext::Error::NotCallable => Self::NotCallable,
            ext::Error::LoggingDisabled => Self::LoggingDisabled,
            ext::Error::EcdsaRecoveryFailed => Self::EcdsaRecoveryFailed,
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
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V) -> Option<u32>
    where
        V: scale::Encode,
    {
        let v = scale::Encode::encode(value);
        self.engine.set_storage(key.as_ref(), &v[..])
    }

    fn get_contract_storage<R>(&mut self, key: &Key) -> Result<Option<R>>
    where
        R: scale::Decode,
    {
        let mut output: [u8; 9600] = [0; 9600];
        match self.engine.get_storage(key.as_ref(), &mut &mut output[..]) {
            Ok(_) => (),
            Err(ext::Error::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = scale::Decode::decode(&mut &output[..])?;
        Ok(Some(decoded))
    }

    fn contract_storage_contains(&mut self, _key: &Key) -> Option<u32> {
        unimplemented!(
            "the off-chain env does not implement `seal_contains_storage`, yet"
        )
    }

    fn clear_contract_storage(&mut self, key: &Key) {
        self.engine.clear_storage(key.as_ref())
    }

    fn set_storage_value<V>(&mut self, key: &StorageKey, value: &V)
    where
        V: scale::Encode,
    {
        let v = scale::Encode::encode(value);
        self.engine
            .set_storage(StorageKeyComposer::old_key(key).as_ref(), &v[..]);
    }

    fn get_storage_value<R>(&mut self, key: &StorageKey) -> Result<Option<R>>
    where
        R: scale::Decode,
    {
        let mut output: [u8; 9600] = [0; 9600];
        match self.engine.get_storage(
            StorageKeyComposer::old_key(key).as_ref(),
            &mut &mut output[..],
        ) {
            Ok(_) => (),
            Err(ext::Error::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = scale::Decode::decode(&mut &output[..])?;
        Ok(Some(decoded))
    }

    fn clear_storage_value(&mut self, key: &StorageKey) {
        self.engine
            .clear_storage(StorageKeyComposer::old_key(key).as_ref())
    }

    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        unimplemented!("the off-chain env does not implement `seal_input`")
    }

    fn return_value<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: scale::Encode,
    {
        unimplemented!("the off-chain env does not implement `seal_return_value`")
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

    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()> {
        use secp256k1::{
            ecdsa::{
                RecoverableSignature,
                RecoveryId,
            },
            Message,
            SECP256K1,
        };

        // In most implementations, the v is just 0 or 1 internally, but 27 was added
        // as an arbitrary number for signing Bitcoin messages and Ethereum adopted that as well.
        let recovery_byte = if signature[64] > 26 {
            signature[64] - 27
        } else {
            signature[64]
        };
        let recovery_id = RecoveryId::from_i32(recovery_byte as i32)
            .unwrap_or_else(|error| panic!("Unable to parse the recovery id: {}", error));
        let message = Message::from_slice(message_hash).unwrap_or_else(|error| {
            panic!("Unable to create the message from hash: {}", error)
        });
        let signature =
            RecoverableSignature::from_compact(&signature[0..64], recovery_id)
                .unwrap_or_else(|error| {
                    panic!("Unable to parse the signature: {}", error)
                });

        let pub_key = SECP256K1.recover_ecdsa(&message, &signature);
        match pub_key {
            Ok(pub_key) => {
                *output = pub_key.serialize();
                Ok(())
            }
            Err(_) => Err(Error::EcdsaRecoveryFailed),
        }
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

        self.engine
            .call_chain_extension(func_id, enc_input, &mut &mut output[..]);
        let (status, out): (u32, Vec<u8>) = scale::Decode::decode(&mut &output[..])
            .unwrap_or_else(|error| {
                panic!(
                    "could not decode `call_chain_extension` output: {:?}",
                    error
                )
            });

        status_to_result(status)?;
        let decoded = decode_to_result(&out[..])?;
        Ok(decoded)
    }

    fn set_code_hash(&mut self, _code_hash: &[u8]) -> Result<()> {
        unimplemented!("off-chain environment does not support `set_code_hash`")
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller<E: Environment>(&mut self) -> E::AccountId {
        self.get_property::<E::AccountId>(Engine::caller)
            .unwrap_or_else(|error| {
                panic!("could not read `caller` property: {:?}", error)
            })
    }

    fn transferred_value<E: Environment>(&mut self) -> E::Balance {
        self.get_property::<E::Balance>(Engine::value_transferred)
            .unwrap_or_else(|error| {
                panic!("could not read `transferred_value` property: {:?}", error)
            })
    }

    fn gas_left<E: Environment>(&mut self) -> u64 {
        self.get_property::<u64>(Engine::gas_left)
            .unwrap_or_else(|error| {
                panic!("could not read `gas_left` property: {:?}", error)
            })
    }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        self.get_property::<E::Timestamp>(Engine::block_timestamp)
            .unwrap_or_else(|error| {
                panic!("could not read `block_timestamp` property: {:?}", error)
            })
    }

    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        self.get_property::<E::AccountId>(Engine::address)
            .unwrap_or_else(|error| {
                panic!("could not read `account_id` property: {:?}", error)
            })
    }

    fn balance<E: Environment>(&mut self) -> E::Balance {
        self.get_property::<E::Balance>(Engine::balance)
            .unwrap_or_else(|error| {
                panic!("could not read `balance` property: {:?}", error)
            })
    }

    fn block_number<E: Environment>(&mut self) -> E::BlockNumber {
        self.get_property::<E::BlockNumber>(Engine::block_number)
            .unwrap_or_else(|error| {
                panic!("could not read `block_number` property: {:?}", error)
            })
    }

    fn minimum_balance<E: Environment>(&mut self) -> E::Balance {
        self.get_property::<E::Balance>(Engine::minimum_balance)
            .unwrap_or_else(|error| {
                panic!("could not read `minimum_balance` property: {:?}", error)
            })
    }

    fn emit_event<E, Event>(&mut self, event: Event)
    where
        E: Environment,
        Event: Topics + scale::Encode,
    {
        let builder = TopicsBuilder::default();
        let enc_topics = event.topics::<E, _>(builder.into());
        let enc_data = &scale::Encode::encode(&event)[..];
        self.engine.deposit_event(&enc_topics[..], enc_data);
    }

    fn invoke_contract<E, Args, R>(
        &mut self,
        params: &CallParams<E, Call<E>, Args, R>,
    ) -> Result<R>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        let _gas_limit = params.gas_limit();
        let _callee = params.callee();
        let _call_flags = params.call_flags().into_u32();
        let _transferred_value = params.transferred_value();
        let _input = params.exec_input();
        unimplemented!("off-chain environment does not support contract invocation")
    }

    fn invoke_contract_delegate<E, Args, R>(
        &mut self,
        params: &CallParams<E, DelegateCall<E>, Args, R>,
    ) -> Result<R>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        let _code_hash = params.code_hash();
        unimplemented!(
            "off-chain environment does not support delegated contract invocation"
        )
    }

    fn instantiate_contract<E, Args, Salt, C>(
        &mut self,
        params: &CreateParams<E, Args, Salt, C>,
    ) -> Result<E::AccountId>
    where
        E: Environment,
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        let _code_hash = params.code_hash();
        let _gas_limit = params.gas_limit();
        let _endowment = params.endowment();
        let _input = params.exec_input();
        let _salt_bytes = params.salt_bytes();
        unimplemented!("off-chain environment does not support contract instantiation")
    }

    fn terminate_contract<E>(&mut self, beneficiary: E::AccountId) -> !
    where
        E: Environment,
    {
        let buffer = scale::Encode::encode(&beneficiary);
        self.engine.terminate(&buffer[..])
    }

    fn transfer<E>(&mut self, destination: E::AccountId, value: E::Balance) -> Result<()>
    where
        E: Environment,
    {
        let enc_destination = &scale::Encode::encode(&destination)[..];
        let enc_value = &scale::Encode::encode(&value)[..];
        self.engine
            .transfer(enc_destination, enc_value)
            .map_err(Into::into)
    }

    fn weight_to_fee<E: Environment>(&mut self, gas: u64) -> E::Balance {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        self.engine.weight_to_fee(gas, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).unwrap_or_else(|error| {
            panic!("could not read `weight_to_fee` property: {:?}", error)
        })
    }

    fn random<E>(&mut self, subject: &[u8]) -> Result<(E::Hash, E::BlockNumber)>
    where
        E: Environment,
    {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        self.engine.random(subject, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).map_err(Into::into)
    }

    fn is_contract<E>(&mut self, _account: &E::AccountId) -> bool
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support contract instantiation")
    }

    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support cross-contract calls")
    }

    fn code_hash<E>(&mut self, _account: &E::AccountId) -> Result<E::Hash>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `code_hash`")
    }

    fn own_code_hash<E>(&mut self) -> Result<E::Hash>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `own_code_hash`")
    }
}
