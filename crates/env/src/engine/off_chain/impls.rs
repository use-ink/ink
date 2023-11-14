// Copyright (C) Parity Technologies (UK) Ltd.
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
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAccountId,
    },
    event::{
        Event,
        TopicsBuilderBackend,
    },
    hash::{
        Blake2x128,
        Blake2x256,
        CryptoHash,
        HashOutput,
        Keccak256,
        Sha2x256,
    },
    types::Environment,
    Clear,
    EnvBackend,
    Error,
    Result,
    ReturnFlags,
    TypedEnvBackend,
};
use ink_engine::{
    ext,
    ext::Engine,
};
use ink_storage_traits::{
    decode_all,
    Storable,
};
use schnorrkel::{
    PublicKey,
    Signature,
};

/// The capacity of the static buffer.
/// This is the same size as the ink! on-chain environment. We chose to use the same size
/// to be as close to the on-chain behavior as possible.
const BUFFER_SIZE: usize = crate::BUFFER_SIZE;

/// Proxy function used to simulate code hash and to invoke contract methods.
fn execute_contract_call<ContractRef>(input: Vec<u8>) -> Vec<u8>
where
    ContractRef: crate::ContractReverseReference,
    <ContractRef as crate::ContractReverseReference>::Type:
        crate::reflect::ContractMessageDecoder,
{
    let dispatch = <
        <
            <
                ContractRef
                as crate::ContractReverseReference
            >::Type
            as crate::reflect::ContractMessageDecoder
        >::Type
        as scale::Decode
    >::decode(&mut &input[..])
        .unwrap_or_else(|e| panic!("Failed to decode constructor call: {:?}", e));

    crate::reflect::ExecuteDispatchable::execute_dispatchable(dispatch)
        .unwrap_or_else(|e| panic!("Message call failed: {:?}", e));

    crate::test::get_return_value()
}

fn invoke_contract_impl<E, R>(
    env: &mut EnvInstance,
    _gas_limit: Option<u64>,
    _call_flags: u32,
    _transferred_value: Option<&<E as Environment>::Balance>,
    callee_account: Option<&<E as Environment>::AccountId>,
    code_hash: Option<&<E as Environment>::Hash>,
    input: Vec<u8>,
) -> Result<ink_primitives::MessageResult<R>>
where
    E: Environment,
    R: scale::Decode,
{
    let mut callee_code_hash = match callee_account {
        Some(ca) => env.code_hash::<E>(ca)?,
        None => *code_hash.unwrap(),
    };

    let handler = env
        .engine
        .database
        .get_contract_message_handler(callee_code_hash.as_mut());
    let old_callee = env.engine.get_callee();
    let mut restore_callee = false;
    if let Some(callee_account) = callee_account {
        let encoded_callee = scale::Encode::encode(callee_account);
        env.engine.set_callee(encoded_callee);
        restore_callee = true;
        env.engine.exec_context.depth += 1;
    }

    let result = handler(input);

    if restore_callee {
        env.engine.set_callee(old_callee);
        env.engine.exec_context.depth -= 1;
    }

    let result =
        <ink_primitives::MessageResult<R> as scale::Decode>::decode(&mut &result[..])
            .expect("failed to decode return value");

    Ok(result)
}

impl CryptoHash for Blake2x128 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 16];
        static_assertions::assert_type_eq_all!(
            <Blake2x128 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 16);
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
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
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
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
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
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
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
            ext::Error::Sr25519VerifyFailed => Self::Sr25519VerifyFailed,
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
        let mut result = <E as Environment>::Hash::CLEAR_HASH;
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

    pub fn get_return_value(&mut self) -> Vec<u8> {
        self.engine.get_storage(&[255_u8; 32]).unwrap().to_vec()
    }

    pub fn upload_code<ContractRef>(&mut self) -> ink_primitives::types::Hash
    where
        ContractRef: crate::ContractReverseReference,
        <ContractRef as crate::ContractReverseReference>::Type:
            crate::reflect::ContractMessageDecoder,
    {
        ink_primitives::types::Hash::from(
            self.engine
                .database
                .set_contract_message_handler(execute_contract_call::<ContractRef>),
        )
    }
}

impl EnvBackend for EnvInstance {
    fn set_contract_storage<K, V>(&mut self, key: &K, value: &V) -> Option<u32>
    where
        K: scale::Encode,
        V: Storable,
    {
        let mut v = vec![];
        Storable::encode(value, &mut v);
        self.engine.set_storage(&key.encode(), &v[..])
    }

    fn get_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        match self.engine.get_storage(&key.encode()) {
            Ok(res) => {
                let decoded = decode_all(&mut &res[..])?;
                Ok(Some(decoded))
            }
            Err(ext::Error::KeyNotFound) => Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
    }

    fn take_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        match self.engine.take_storage(&key.encode()) {
            Ok(output) => {
                let decoded = decode_all(&mut &output[..])?;
                Ok(Some(decoded))
            }
            Err(ext::Error::KeyNotFound) => Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
    }

    fn contains_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        self.engine.contains_storage(&key.encode())
    }

    fn clear_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        self.engine.clear_storage(&key.encode())
    }

    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    #[cfg(not(feature = "test_instantiate"))]
    fn return_value<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: scale::Encode,
    {
        panic!("enable feature test_instantiate to use return_value()")
    }

    #[cfg(feature = "test_instantiate")]
    fn return_value<R>(&mut self, _flags: ReturnFlags, return_value: &R)
    where
        R: scale::Encode,
    {
        let mut v = vec![];
        return_value.encode_to(&mut v);
        self.engine.set_storage(&[255_u8; 32], &v[..]);
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
        // as an arbitrary number for signing Bitcoin messages and Ethereum adopted that
        // as well.
        let recovery_byte = if signature[64] > 26 {
            signature[64] - 27
        } else {
            signature[64]
        };
        let recovery_id = RecoveryId::from_i32(recovery_byte as i32)
            .unwrap_or_else(|error| panic!("Unable to parse the recovery id: {error}"));
        let message = Message::from_digest_slice(message_hash).unwrap_or_else(|error| {
            panic!("Unable to create the message from hash: {error}")
        });
        let signature =
            RecoverableSignature::from_compact(&signature[0..64], recovery_id)
                .unwrap_or_else(|error| panic!("Unable to parse the signature: {error}"));

        let pub_key = SECP256K1.recover_ecdsa(&message, &signature);
        match pub_key {
            Ok(pub_key) => {
                *output = pub_key.serialize();
                Ok(())
            }
            Err(_) => Err(Error::EcdsaRecoveryFailed),
        }
    }

    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
    ) -> Result<()> {
        let pk = secp256k1::PublicKey::from_slice(pubkey)
            .map_err(|_| Error::EcdsaRecoveryFailed)?;
        let uncompressed = pk.serialize_uncompressed();
        let mut hash = <Keccak256 as HashOutput>::Type::default();
        <Keccak256>::hash(&uncompressed[1..], &mut hash);
        output.as_mut().copy_from_slice(&hash[12..]);
        Ok(())
    }

    fn sr25519_verify(
        &mut self,
        signature: &[u8; 64],
        message: &[u8],
        pub_key: &[u8; 32],
    ) -> Result<()> {
        // the context associated with the signing (specific to the sr25519 algorithm)
        // defaults to "substrate" in substrate, but could be different elsewhere
        // https://github.com/paritytech/substrate/blob/c32f5ed2ae6746d6f791f08cecbfc22fa188f5f9/primitives/core/src/sr25519.rs#L60
        let context = b"substrate";
        // attempt to parse a signature from bytes
        let signature: Signature =
            Signature::from_bytes(signature).map_err(|_| Error::Sr25519VerifyFailed)?;
        // attempt to parse a public key from bytes
        let public_key: PublicKey =
            PublicKey::from_bytes(pub_key).map_err(|_| Error::Sr25519VerifyFailed)?;
        // verify the signature
        public_key
            .verify_simple(context, message, &signature)
            .map_err(|_| Error::Sr25519VerifyFailed)
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
                panic!("could not decode `call_chain_extension` output: {error:?}")
            });

        status_to_result(status)?;
        let decoded = decode_to_result(&out[..])?;
        Ok(decoded)
    }

    fn set_code_hash(&mut self, code_hash: &[u8]) -> Result<()> {
        self.engine
            .database
            .set_code_hash(&self.engine.get_callee(), code_hash);
        Ok(())
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller<E: Environment>(&mut self) -> E::AccountId {
        self.get_property::<E::AccountId>(Engine::caller)
            .unwrap_or_else(|error| panic!("could not read `caller` property: {error:?}"))
    }

    fn transferred_value<E: Environment>(&mut self) -> E::Balance {
        self.get_property::<E::Balance>(Engine::value_transferred)
            .unwrap_or_else(|error| {
                panic!("could not read `transferred_value` property: {error:?}")
            })
    }

    fn gas_left<E: Environment>(&mut self) -> u64 {
        self.get_property::<u64>(Engine::gas_left)
            .unwrap_or_else(|error| {
                panic!("could not read `gas_left` property: {error:?}")
            })
    }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        self.get_property::<E::Timestamp>(Engine::block_timestamp)
            .unwrap_or_else(|error| {
                panic!("could not read `block_timestamp` property: {error:?}")
            })
    }

    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        self.get_property::<E::AccountId>(Engine::address)
            .unwrap_or_else(|error| {
                panic!("could not read `account_id` property: {error:?}")
            })
    }

    fn balance<E: Environment>(&mut self) -> E::Balance {
        self.get_property::<E::Balance>(Engine::balance)
            .unwrap_or_else(|error| {
                panic!("could not read `balance` property: {error:?}")
            })
    }

    fn block_number<E: Environment>(&mut self) -> E::BlockNumber {
        self.get_property::<E::BlockNumber>(Engine::block_number)
            .unwrap_or_else(|error| {
                panic!("could not read `block_number` property: {error:?}")
            })
    }

    fn minimum_balance<E: Environment>(&mut self) -> E::Balance {
        self.get_property::<E::Balance>(Engine::minimum_balance)
            .unwrap_or_else(|error| {
                panic!("could not read `minimum_balance` property: {error:?}")
            })
    }

    fn emit_event<E, Evt>(&mut self, event: Evt)
    where
        E: Environment,
        Evt: Event,
    {
        let builder = TopicsBuilder::default();
        let enc_topics = event.topics::<E, _>(builder.into());
        let enc_data = &scale::Encode::encode(&event)[..];
        self.engine.deposit_event(&enc_topics[..], enc_data);
    }

    fn invoke_contract<E, Args, R>(
        &mut self,
        params: &CallParams<E, Call<E>, Args, R>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        let gas_limit = params.gas_limit();
        let call_flags = params.call_flags().into_u32();
        let transferred_value = params.transferred_value();
        let input = params.exec_input();
        let callee_account = params.callee();
        let input = scale::Encode::encode(input);

        invoke_contract_impl::<E, R>(
            self,
            Some(gas_limit),
            call_flags,
            Some(transferred_value),
            Some(callee_account),
            None,
            input,
        )
    }

    fn invoke_contract_delegate<E, Args, R>(
        &mut self,
        params: &CallParams<E, DelegateCall<E>, Args, R>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        let call_flags = params.call_flags().into_u32();
        let input = params.exec_input();
        let code_hash = params.code_hash();
        let input = scale::Encode::encode(input);

        invoke_contract_impl::<E, R>(
            self,
            None,
            call_flags,
            None,
            None,
            Some(code_hash),
            input,
        )
    }

    fn instantiate_contract<E, ContractRef, Args, Salt, R>(
        &mut self,
        params: &CreateParams<E, ContractRef, Args, Salt, R>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <R as ConstructorReturnType<ContractRef>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAccountId<E> + crate::ContractReverseReference,
        <ContractRef as crate::ContractReverseReference>::Type:
            crate::reflect::ContractConstructorDecoder,
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
        R: ConstructorReturnType<ContractRef>,
    {
        let _gas_limit = params.gas_limit();

        let endowment = params.endowment();
        let endowment = scale::Encode::encode(endowment);
        let endowment: u128 = scale::Decode::decode(&mut &endowment[..])?;

        let salt_bytes = params.salt_bytes();

        let code_hash = params.code_hash();
        let code_hash = scale::Encode::encode(code_hash);

        let input = params.exec_input();
        let input = scale::Encode::encode(input);

        // Compute account for instantiated contract.
        let account_id_vec = {
            let mut account_input = Vec::<u8>::new();
            account_input.extend(&b"contract_addr_v1".to_vec());
            if let Some(caller) = &self.engine.exec_context.caller {
                scale::Encode::encode_to(&caller.as_bytes(), &mut account_input);
            }
            account_input.extend(&code_hash);
            account_input.extend(&input);
            account_input.extend(salt_bytes.as_ref());
            let mut account_id = [0_u8; 32];
            ink_engine::hashing::blake2b_256(&account_input[..], &mut account_id);
            account_id.to_vec()
        };

        let mut account_id =
            <E as Environment>::AccountId::decode(&mut &account_id_vec[..]).unwrap();

        let old_callee = self.engine.get_callee();
        self.engine.exec_context.depth += 1;
        self.engine.set_callee(account_id_vec.clone());

        let dispatch = <
            <
                <
                    ContractRef
                    as crate::ContractReverseReference
                >::Type
                as crate::reflect::ContractConstructorDecoder
            >::Type
            as scale::Decode
        >::decode(&mut &input[..])
            .unwrap_or_else(|e| panic!("Failed to decode constructor call: {:?}", e));
        crate::reflect::ExecuteDispatchable::execute_dispatchable(dispatch)
            .unwrap_or_else(|e| panic!("Constructor call failed: {:?}", e));

        self.set_code_hash(code_hash.as_slice())?;
        self.engine.set_contract(account_id_vec.clone());
        self.engine
            .database
            .set_balance(account_id.as_mut(), endowment);

        self.engine.set_callee(old_callee);
        self.engine.exec_context.depth -= 1;

        Ok(Ok(R::ok(
            <ContractRef as FromAccountId<E>>::from_account_id(account_id),
        )))
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
            panic!("could not read `weight_to_fee` property: {error:?}")
        })
    }

    fn is_contract<E>(&mut self, account: &E::AccountId) -> bool
    where
        E: Environment,
    {
        self.engine.is_contract(scale::Encode::encode(&account))
    }

    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment,
    {
        self.engine.caller_is_origin()
    }

    fn code_hash<E>(&mut self, account: &E::AccountId) -> Result<E::Hash>
    where
        E: Environment,
    {
        let code_hash = self
            .engine
            .database
            .get_code_hash(&scale::Encode::encode(&account));
        if let Some(code_hash) = code_hash {
            let code_hash =
                <E as Environment>::Hash::decode(&mut &code_hash[..]).unwrap();
            Ok(code_hash)
        } else {
            Err(Error::KeyNotFound)
        }
    }

    fn own_code_hash<E>(&mut self) -> Result<E::Hash>
    where
        E: Environment,
    {
        let callee = &self.engine.get_callee();
        let code_hash = self.engine.database.get_code_hash(callee);
        if let Some(code_hash) = code_hash {
            let code_hash =
                <E as Environment>::Hash::decode(&mut &code_hash[..]).unwrap();
            Ok(code_hash)
        } else {
            Err(Error::KeyNotFound)
        }
    }

    fn call_runtime<E, Call>(&mut self, _call: &Call) -> Result<()>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `call_runtime`")
    }
}
