// Copyright (C) Use Ink (UK) Ltd.
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

use ink_engine::ext::Engine;
use ink_primitives::{
    Address,
    H256,
    U256,
    abi::{
        AbiEncodeWith,
        Ink,
        Sol,
    },
    sol::SolResultEncode,
    types::{
        AccountIdMapper,
        Environment,
    },
};
use ink_storage_traits::{
    Storable,
    decode_all,
};
use pallet_revive_uapi::{
    ReturnErrorCode,
    ReturnFlags,
};
#[cfg(feature = "unstable-hostfn")]
use schnorrkel::{
    PublicKey,
    Signature,
};

use super::EnvInstance;
use crate::{
    DecodeDispatch,
    DispatchError,
    EnvBackend,
    Result,
    TypedEnvBackend,
    call::{
        Call,
        CallParams,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAddr,
        LimitParamsV2,
        utils::DecodeMessageResult,
    },
    event::{
        Event,
        TopicEncoder,
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
    test::callee,
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
        as DecodeDispatch
    >::decode_dispatch(&mut &input[..])
        .unwrap_or_else(|e| panic!("Failed to decode constructor call: {e:?}"));

    crate::reflect::ExecuteDispatchable::execute_dispatchable(dispatch)
        .unwrap_or_else(|e| panic!("Message call failed: {e:?}"));

    crate::test::get_return_value()
}

fn invoke_contract_impl<R, Abi>(
    env: &mut EnvInstance,
    _gas_limit: Option<u64>,
    _call_flags: u32,
    _transferred_value: Option<&U256>,
    callee_account: Address,
    input: Vec<u8>,
) -> Result<ink_primitives::MessageResult<R>>
where
    R: DecodeMessageResult<Abi>,
{
    let callee_code_hash = env.code_hash(&callee_account).unwrap_or_else(|err| {
        panic!("failed getting code hash for {callee_account:?}: {err:?}")
    });

    let handler = env
        .engine
        .database
        .get_contract_message_handler(&callee_code_hash);
    let old_callee = env.engine.get_callee();
    env.engine.set_callee(callee_account);

    let result = handler(input);

    env.engine.set_callee(old_callee);

    // TODO: (@davidsemakula) Track return flag and set `did_revert` as appropriate.
    R::decode_output(&result, false)
}

fn invoke_contract_impl_delegate<R, Abi>(
    env: &mut EnvInstance,
    _gas_limit: Option<u64>,
    _call_flags: u32,
    _transferred_value: Option<&U256>,
    callee_account: Address,
    input: Vec<u8>,
) -> Result<ink_primitives::MessageResult<R>>
where
    R: DecodeMessageResult<Abi>,
{
    let callee_code_hash = env.code_hash(&callee_account).unwrap_or_else(|err| {
        panic!("failed getting code hash for {callee_account:?}: {err:?}")
    });

    let handler = env
        .engine
        .database
        .get_contract_message_handler(&callee_code_hash);
    let result = handler(input);

    // TODO: (@davidsemakula) Track return flag and set `did_revert` as appropriate.
    R::decode_output(&result, false)
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

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        unreachable!("not required, `hash` has been implemented.");
    }
}

impl CryptoHash for Blake2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        // For the engine (simulation of `pallet-revive` in `std`) we skip on the buffer
        // in the implementation for simplicity.
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Blake2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
        Engine::hash_blake2_256(input, output);
    }

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        unreachable!("not required, `hash` has been implemented.");
    }
}

impl CryptoHash for Sha2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        // For the engine (simulation of `pallet-revive` in `std`) we skip on the buffer
        // in the implementation for simplicity.
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Sha2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
        Engine::hash_sha2_256(input, output);
    }

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        unreachable!("not required, `hash` has been implemented.");
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

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        unreachable!("not required, `hash` has been implemented.");
    }
}

#[derive(Default)]
pub struct TopicsBuilder {
    pub topics: Vec<[u8; 32]>,
}

impl<Abi> TopicsBuilderBackend<Abi> for TopicsBuilder
where
    Abi: TopicEncoder,
{
    type Output = Vec<[u8; 32]>;

    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: AbiEncodeWith<Abi>,
    {
        let output = <Abi as TopicEncoder>::encode_topic(topic_value);
        self.topics.push(output);
    }

    fn output(self) -> Self::Output {
        self.topics
    }
}

impl TopicEncoder for Ink {
    const REQUIRES_BUFFER: bool = false;

    fn encode_topic<T>(value: &T) -> [u8; 32]
    where
        T: AbiEncodeWith<Self>,
    {
        value.encode_topic(<Blake2x256 as CryptoHash>::hash)
    }

    fn encode_topic_with_hash_buffer<T>(
        _value: &T,
        _output: &mut [u8; 32],
        _buffer: &mut [u8],
    ) where
        T: AbiEncodeWith<Self>,
    {
        unreachable!("not required, `encode_topic` has been implemented.");
    }
}

impl TopicEncoder for Sol {
    const REQUIRES_BUFFER: bool = false;

    fn encode_topic<T>(value: &T) -> [u8; 32]
    where
        T: AbiEncodeWith<Self>,
    {
        value.encode_topic(<Keccak256 as CryptoHash>::hash)
    }

    fn encode_topic_with_hash_buffer<T>(
        _value: &T,
        _output: &mut [u8; 32],
        _buffer: &mut [u8],
    ) where
        T: AbiEncodeWith<Self>,
    {
        unreachable!("not required, `encode_topic` has been implemented.");
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

    pub fn upload_code<ContractRef>(&mut self) -> H256
    where
        ContractRef: crate::ContractReverseReference,
        <ContractRef as crate::ContractReverseReference>::Type:
            crate::reflect::ContractMessageDecoder,
    {
        H256::from(
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
            Err(ReturnErrorCode::KeyNotFound) => Ok(None),
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
            Err(ReturnErrorCode::KeyNotFound) => Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
    }

    fn remaining_buffer(&mut self) -> usize {
        BUFFER_SIZE
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

    fn decode_input<T>(&mut self) -> core::result::Result<T, DispatchError>
    where
        T: DecodeDispatch,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    #[cfg(not(feature = "std"))]
    fn return_value<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: scale::Encode,
    {
        panic!("enable feature `std` to use `return_value()`")
    }

    #[cfg(feature = "std")]
    fn return_value<R>(&mut self, _flags: ReturnFlags, return_value: &R)
    where
        R: scale::Encode,
    {
        let mut v = vec![];
        return_value.encode_to(&mut v);
        self.engine.set_storage(&[255_u8; 32], &v[..]);
    }

    fn return_value_solidity<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: for<'a> SolResultEncode<'a>,
    {
        unimplemented!("the off-chain env does not implement `return_value_solidity`")
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

    #[allow(clippy::arithmetic_side_effects)] // todo
    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()> {
        use secp256k1::{
            Message,
            SECP256K1,
            ecdsa::{
                RecoverableSignature,
                RecoveryId,
            },
        };

        // In most implementations, the v is just 0 or 1 internally, but 27 was added
        // as an arbitrary number for signing Bitcoin messages and Ethereum adopted that
        // as well.
        let recovery_byte = if signature[64] > 26 {
            signature[64] - 27
        } else {
            signature[64]
        };
        let recovery_id = RecoveryId::try_from(recovery_byte as i32)
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
            Err(_) => Err(ReturnErrorCode::EcdsaRecoveryFailed.into()),
        }
    }

    #[cfg(feature = "unstable-hostfn")]
    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
    ) -> Result<()> {
        let pk = secp256k1::PublicKey::from_slice(pubkey)
            .map_err(|_| ReturnErrorCode::EcdsaRecoveryFailed)?;
        let uncompressed = pk.serialize_uncompressed();
        let mut hash = <Keccak256 as HashOutput>::Type::default();
        <Keccak256>::hash(&uncompressed[1..], &mut hash);
        output.as_mut().copy_from_slice(&hash[12..]);
        Ok(())
    }

    #[cfg(feature = "unstable-hostfn")]
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
        let signature: Signature = Signature::from_bytes(signature)
            .map_err(|_| ReturnErrorCode::Sr25519VerifyFailed)?;
        // attempt to parse a public key from bytes
        let public_key: PublicKey = PublicKey::from_bytes(pub_key)
            .map_err(|_| ReturnErrorCode::Sr25519VerifyFailed)?;
        // verify the signature
        public_key
            .verify_simple(context, message, &signature)
            .map_err(|_| ReturnErrorCode::Sr25519VerifyFailed.into())
    }

    #[cfg(feature = "unstable-hostfn")]
    fn set_code_hash(&mut self, code_hash: &H256) -> Result<()> {
        self.engine
            .database
            .set_code_hash(&self.engine.get_callee(), code_hash);
        Ok(())
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller(&mut self) -> Address {
        self.get_property::<Address>(Engine::caller)
            .unwrap_or_else(|error| panic!("could not read `caller` property: {error:?}"))
    }

    fn transferred_value(&mut self) -> U256 {
        self.get_property(Engine::value_transferred)
            .unwrap_or_else(|error| {
                panic!("could not read `transferred_value` property: {error:?}")
            })
    }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        self.get_property::<E::Timestamp>(Engine::block_timestamp)
            .unwrap_or_else(|error| {
                panic!("could not read `block_timestamp` property: {error:?}")
            })
    }

    #[cfg(feature = "unstable-hostfn")]
    fn to_account_id<E: Environment>(&mut self, addr: Address) -> E::AccountId {
        let mut full_scope: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let full_scope = &mut &mut full_scope[..];
        Engine::to_account_id(&self.engine, addr.as_bytes(), full_scope);
        scale::Decode::decode(&mut &full_scope[..]).unwrap()
    }

    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        self.get_property::<E::AccountId>(Engine::account_id)
            .unwrap_or_else(|error| {
                panic!("could not read `account_id` property: {error:?}")
            })
    }

    fn address(&mut self) -> Address {
        self.get_property::<Address>(Engine::address)
            .unwrap_or_else(|error| {
                panic!("could not read `account_id` property: {error:?}")
            })
    }

    fn balance(&mut self) -> U256 {
        self.get_property::<U256>(Engine::balance)
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

    fn minimum_balance(&mut self) -> U256 {
        self.get_property::<U256>(Engine::minimum_balance)
            .unwrap_or_else(|error| {
                panic!("could not read `minimum_balance` property: {error:?}")
            })
    }

    fn emit_event<Evt, Abi>(&mut self, event: &Evt)
    where
        Evt: Event<Abi>,
        Abi: TopicEncoder,
    {
        let builder = TopicsBuilder::default();
        let enc_topics = event.topics(builder.into());
        let enc_data = event.encode_data();
        self.engine
            .deposit_event(&enc_topics[..], enc_data.as_slice());
    }

    fn invoke_contract<E, Args, R, Abi>(
        &mut self,
        params: &CallParams<E, Call, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: AbiEncodeWith<Abi>,
        R: DecodeMessageResult<Abi>,
    {
        let call_flags = params.call_flags().bits();
        let transferred_value = params.transferred_value();
        let input = params.exec_input().encode();
        let callee_account = params.callee();

        invoke_contract_impl::<R, Abi>(
            self,
            None,
            call_flags,
            Some(transferred_value),
            *callee_account, // todo possibly return no reference from callee()
            input,
        )
    }

    fn invoke_contract_delegate<E, Args, R, Abi>(
        &mut self,
        params: &CallParams<E, DelegateCall, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: AbiEncodeWith<Abi>,
        R: DecodeMessageResult<Abi>,
    {
        let _addr = params.address(); // todo remove
        let call_flags = params.call_flags().bits();
        let input = params.exec_input().encode();

        invoke_contract_impl_delegate::<R, Abi>(
            self,
            None,
            call_flags,
            None,
            *params.address(),
            input,
        )
    }

    fn instantiate_contract<E, ContractRef, Args, R, Abi>(
        &mut self,
        params: &CreateParams<E, ContractRef, LimitParamsV2, Args, R, Abi>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <R as ConstructorReturnType<ContractRef, Abi>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAddr + crate::ContractReverseReference,
        <ContractRef as crate::ContractReverseReference>::Type:
            crate::reflect::ContractConstructorDecoder,
        Args: AbiEncodeWith<Abi>,
        R: ConstructorReturnType<ContractRef, Abi>,
    {
        let endowment = params.endowment();
        let salt_bytes = params.salt_bytes();
        let code_hash = params.code_hash();

        let input = params.exec_input().encode();

        // Compute address for instantiated contract.
        let account_id_vec = {
            let mut account_input = Vec::<u8>::new();
            account_input.extend(&b"contract_addr".to_vec());
            scale::Encode::encode_to(
                &self.engine.exec_context.caller.as_bytes(),
                &mut account_input,
            );
            account_input.extend(&code_hash.0);
            account_input.extend(&input);
            if let Some(salt) = salt_bytes {
                account_input.extend(salt);
            }
            let mut account_id = [0_u8; 32];
            ink_engine::hashing::blake2b_256(&account_input[..], &mut account_id);
            account_id.to_vec()
        };
        // todo don't convert to vec and back, simplify type
        let contract_addr = AccountIdMapper::to_address(&account_id_vec[..]);

        let old_callee = self.engine.get_callee();
        self.engine.set_callee(contract_addr);

        let dispatch = <
            <
                <
                    ContractRef
                    as crate::ContractReverseReference
                >::Type
                as crate::reflect::ContractConstructorDecoder
            >::Type
            as DecodeDispatch
        >::decode_dispatch(&mut &input[..])
            .unwrap_or_else(|e| panic!("Failed to decode constructor call: {e:?}"));
        crate::reflect::ExecuteDispatchable::execute_dispatchable(dispatch)
            .unwrap_or_else(|e| panic!("Constructor call failed: {e:?}"));

        self.engine
            .database
            .set_code_hash(&self.engine.get_callee(), code_hash);
        self.engine.set_contract(callee());
        self.engine
            .database
            // todo passing the types instead of refs would be better
            .set_balance(&callee(), *endowment);

        // todo why?
        self.engine.set_callee(old_callee);

        Ok(Ok(R::ok(<ContractRef as FromAddr>::from_addr(
            contract_addr,
        ))))
    }

    #[cfg(feature = "unstable-hostfn")]
    fn terminate_contract(&mut self, beneficiary: Address) -> ! {
        self.engine.terminate(beneficiary)
    }

    fn transfer<E>(&mut self, destination: Address, value: U256) -> Result<()>
    where
        E: Environment,
    {
        let enc_value = &scale::Encode::encode(&value)[..];
        self.engine
            .transfer(destination, enc_value)
            .map_err(Into::into)
    }

    fn weight_to_fee<E: Environment>(&mut self, gas: u64) -> E::Balance {
        let mut output: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        self.engine.weight_to_fee(gas, &mut &mut output[..]);
        scale::Decode::decode(&mut &output[..]).unwrap_or_else(|error| {
            panic!("could not read `weight_to_fee` property: {error:?}")
        })
    }

    #[cfg(feature = "unstable-hostfn")]
    fn is_contract(&mut self, account: &Address) -> bool {
        self.engine.is_contract(account)
    }

    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support cross-contract calls")
    }

    fn caller_is_root<E>(&mut self) -> bool
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `caller_is_root`")
    }

    fn code_hash(&mut self, addr: &Address) -> Result<H256> {
        let code_hash = self.engine.database.get_code_hash(addr);
        if let Some(code_hash) = code_hash {
            // todo
            let code_hash = H256::decode(&mut &code_hash[..]).unwrap();
            Ok(code_hash)
        } else {
            Err(ReturnErrorCode::KeyNotFound.into())
        }
    }

    fn own_code_hash(&mut self) -> Result<H256> {
        let callee = &self.engine.get_callee();
        let code_hash = self.engine.database.get_code_hash(callee);
        if let Some(code_hash) = code_hash {
            Ok(code_hash)
        } else {
            Err(ReturnErrorCode::KeyNotFound.into())
        }
    }

    #[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
    fn xcm_execute<E, Call>(&mut self, _msg: &xcm::VersionedXcm<Call>) -> Result<()>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `xcm_execute`")
    }

    #[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
    fn xcm_send<E, Call>(
        &mut self,
        _dest: &xcm::VersionedLocation,
        _msg: &xcm::VersionedXcm<Call>,
    ) -> Result<xcm::v4::XcmHash>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `xcm_send`")
    }
}
