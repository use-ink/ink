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

#[cfg(feature = "unstable-hostfn")]
use ink_primitives::abi::Ink;
use ink_primitives::{
    abi::{
        AbiDecodeWith,
        AbiEncodeWith,
    },
    Address,
    SolEncode,
    H256,
    U256,
};
use ink_storage_traits::{
    decode_all,
    Storable,
};
use pallet_revive_uapi::{
    CallFlags,
    HostFn,
    HostFnImpl as ext,
    ReturnErrorCode,
    ReturnFlags,
    StorageFlags,
};
#[cfg(feature = "unstable-hostfn")]
use xcm::VersionedXcm;

use crate::{
    call::{
        utils::DecodeMessageResult,
        Call,
        CallParams,
        DelegateCall,
    },
    engine::on_chain::{
        EncodeScope,
        EnvInstance,
        ScopedBuffer,
    },
    event::{
        Event,
        TopicsBuilderBackend,
    },
    hash::{
        CryptoHash,
        HashOutput,
        Keccak256,
        Sha2x256,
    },
    types::FromLittleEndian,
    DecodeDispatch,
    DispatchError,
    EnvBackend,
    Environment,
    Result,
    TypedEnvBackend,
};
#[cfg(feature = "unstable-hostfn")]
use crate::{
    call::{
        ConstructorReturnType,
        CreateParams,
        FromAddr,
        LimitParamsV2,
    },
    hash::{
        Blake2x128,
        Blake2x256,
    },
    Clear,
};

#[cfg(feature = "unstable-hostfn")]
impl CryptoHash for Blake2x128 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 16];
        static_assertions::assert_type_eq_all!(
            <Blake2x128 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 16);
        ext::hash_blake2_128(input, output);
    }
}

#[cfg(feature = "unstable-hostfn")]
impl CryptoHash for Blake2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Blake2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
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
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000002");
        // todo return value?
        let _ = ext::call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX, /* How much ref_time to devote for the execution. u64::MAX = use
                       * all. */
            u64::MAX, /* How much proof_size to devote for the execution. u64::MAX =
                       * use all. */
            &[u8::MAX; 32],                   // No deposit limit.
            &U256::zero().to_little_endian(), // Value transferred to the contract.
            input,
            Some(&mut &mut output[..]),
        );
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
        ext::hash_keccak_256(input, output);
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

    #[cfg(feature = "unstable-hostfn")]
    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: scale::Encode,
    {
        fn inner<E: Environment>(encoded: &mut [u8]) -> <E as Environment>::Hash {
            let len_encoded = encoded.len();
            let mut result = <E as Environment>::Hash::CLEAR_HASH;
            let len_result = result.as_ref().len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(encoded);
            } else {
                let mut hash_output = <Blake2x256 as HashOutput>::Type::default();
                <Blake2x256 as CryptoHash>::hash(encoded, &mut hash_output);
                let copy_len = core::cmp::min(hash_output.len(), len_result);
                result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            }
            result
        }

        let mut split = self.scoped_buffer.split();
        let encoded = split.take_encoded(topic_value);
        let result = inner::<E>(encoded);
        self.scoped_buffer.append_encoded(&result);
    }

    fn output(mut self) -> Self::Output {
        let encoded_topics = self.scoped_buffer.take_appended();
        (self.scoped_buffer, encoded_topics)
    }
}

impl EnvInstance {
    #[inline(always)]
    /// Returns a new scoped buffer for the entire scope of the static 16 kB buffer.
    fn scoped_buffer(&mut self) -> ScopedBuffer {
        ScopedBuffer::from(&mut self.buffer[..])
    }

    /// Returns the contract property value from its little-endian representation.
    ///
    /// # Note
    ///
    /// This skips the potentially costly decoding step that is often equivalent to a
    /// `memcpy`.
    #[inline(always)]
    fn get_property_little_endian<T>(&mut self, ext_fn: fn(output: &mut [u8; 32])) -> T
    where
        T: FromLittleEndian,
    {
        let mut scope = self.scoped_buffer();
        // TODO: check unwrap
        let u256: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        ext_fn(u256);
        let mut result = <T as FromLittleEndian>::Bytes::default();
        let len = result.as_ref().len();
        result.as_mut()[..].copy_from_slice(&u256[..len]);
        <T as FromLittleEndian>::from_le_bytes(result)
    }
}

const STORAGE_FLAGS: StorageFlags = StorageFlags::empty();

impl EnvBackend for EnvInstance {
    fn set_contract_storage<K, V>(&mut self, key: &K, value: &V) -> Option<u32>
    where
        K: scale::Encode,
        V: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        let value = buffer.take_storable_encoded(value);
        ext::set_storage(STORAGE_FLAGS, key, value)
    }

    fn get_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        let output = &mut buffer.take_rest();
        match ext::get_storage(STORAGE_FLAGS, key, output) {
            Ok(_) => (),
            Err(ReturnErrorCode::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = decode_all(&mut &output[..])?;
        Ok(Some(decoded))
    }

    #[cfg(feature = "unstable-hostfn")]
    fn take_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        let output = &mut buffer.take_rest();
        match ext::take_storage(STORAGE_FLAGS, key, output) {
            Ok(_) => (),
            Err(ReturnErrorCode::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = decode_all(&mut &output[..])?;
        Ok(Some(decoded))
    }

    #[cfg(feature = "unstable-hostfn")]
    fn contains_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        ext::contains_storage(STORAGE_FLAGS, key)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn clear_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        ext::clear_storage(STORAGE_FLAGS, key)
    }

    fn decode_input<T>(&mut self) -> core::result::Result<T, DispatchError>
    where
        T: DecodeDispatch,
    {
        let full_scope = &mut self.scoped_buffer().take_rest();
        ext::call_data_copy(full_scope, 0);
        DecodeDispatch::decode_dispatch(&mut &full_scope[..])
    }

    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: scale::Encode,
    {
        let mut scope = EncodeScope::from(&mut self.buffer[..]);
        return_value.encode_to(&mut scope);
        let len = scope.len();
        ext::return_value(flags, &self.buffer[..][..len]);
    }

    fn return_value_solidity<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: for<'a> SolEncode<'a>,
    {
        let encoded = return_value.encode();
        ext::return_value(flags, &encoded[..]);
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

    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()> {
        // todo change fn args to just take the slice callee_input slice directly
        let mut callee_input = [0u8; 65 + 32];
        callee_input[..65].copy_from_slice(&signature[..65]);
        callee_input[65..65 + 32].copy_from_slice(&message_hash[..32]);

        const ECRECOVER: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000001");
        // todo return value?
        let _ = ext::call(
            CallFlags::empty(),
            &ECRECOVER,
            u64::MAX, /* How much ref_time to devote for the execution. u64::MAX = use
                       * all. */
            u64::MAX, /* How much proof_size to devote for the execution. u64::MAX =
                       * use all. */
            &[u8::MAX; 32],                   // No deposit limit.
            &U256::zero().to_little_endian(), // Value transferred to the contract.
            &callee_input[..],
            Some(&mut &mut output[..]),
        );
        Ok(())
    }

    #[cfg(feature = "unstable-hostfn")]
    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
    ) -> Result<()> {
        ext::ecdsa_to_eth_address(pubkey, output).map_err(Into::into)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn sr25519_verify(
        &mut self,
        signature: &[u8; 64],
        message: &[u8],
        pub_key: &[u8; 32],
    ) -> Result<()> {
        ext::sr25519_verify(signature, message, pub_key).map_err(Into::into)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn call_chain_extension<I, T, E, ErrorCode, F, D>(
        &mut self,
        id: u32,
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
        status_to_result(ext::call_chain_extension(id, enc_input, Some(output)))?;
        let decoded = decode_to_result(output)?;
        Ok(decoded)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn set_code_hash(&mut self, code_hash: &H256) -> Result<()> {
        ext::set_code_hash(code_hash.as_fixed_bytes());
        Ok(()) // todo
    }
}

// TODO remove anything with hash
impl TypedEnvBackend for EnvInstance {
    fn caller(&mut self) -> Address {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::caller(h160);
        // TODO: check decode, I think this could just be from_le_bytes
        scale::Decode::decode(&mut &h160[..])
            .expect("The executed contract must have a caller with a valid account id.")
    }

    fn transferred_value(&mut self) -> U256 {
        let mut scope = self.scoped_buffer();
        let u256: &mut [u8; 32] = scope.take(32).try_into().unwrap();

        ext::value_transferred(u256);
        U256::from_le_bytes(*u256)
    }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        self.get_property_little_endian::<E::Timestamp>(ext::now)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::address(h160);

        let account_id: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        ext::to_account_id(h160, account_id);
        scale::Decode::decode(&mut &account_id[..])
            .expect("A contract being executed must have a valid account id.")
    }

    fn address(&mut self) -> Address {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::address(h160);

        scale::Decode::decode(&mut &h160[..])
            .expect("A contract being executed must have a valid address.")
    }

    fn balance(&mut self) -> U256 {
        self.get_property_little_endian(ext::balance)
    }

    fn block_number<E: Environment>(&mut self) -> E::BlockNumber {
        self.get_property_little_endian::<E::BlockNumber>(ext::block_number)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn minimum_balance<E: Environment>(&mut self) -> E::Balance {
        self.get_property_little_endian::<E::Balance>(ext::minimum_balance)
    }

    fn emit_event<E, Evt>(&mut self, event: Evt)
    where
        E: Environment,
        Evt: Event,
    {
        let (mut scope, enc_topics) =
            event.topics::<E, _>(TopicsBuilder::from(self.scoped_buffer()).into());
        // TODO: improve
        let enc_topics = enc_topics
            .chunks_exact(32)
            .map(|c| c.try_into().unwrap())
            .collect::<ink_prelude::vec::Vec<[u8; 32]>>();
        let enc_data = scope.take_encoded(&event);

        ext::deposit_event(&enc_topics[..], enc_data);
    }

    fn invoke_contract<E, Args, R, Abi>(
        &mut self,
        params: &CallParams<E, Call, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: AbiEncodeWith<Abi>,
        R: AbiDecodeWith<Abi> + DecodeMessageResult<Abi>,
    {
        let mut scope = self.scoped_buffer();
        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let storage_deposit_limit = params.storage_deposit_limit();
        /*
        .map(|limit| {
            let mut enc_storage_limit = EncodeScope::from(scope.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: &mut [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        });
         */
        let enc_storage_limit = to_u256(&mut scope, storage_deposit_limit);

        let enc_callee: &[u8; 20] = params.callee().as_ref().try_into().unwrap();
        let mut enc_transferred_value = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&params.transferred_value(), &mut enc_transferred_value);
        let enc_transferred_value: &mut [u8; 32] =
            enc_transferred_value.into_buffer().try_into().unwrap();
        let call_flags = params.call_flags();
        let enc_input = if !call_flags.contains(CallFlags::FORWARD_INPUT)
            && !call_flags.contains(CallFlags::CLONE_INPUT)
        {
            scope.take_encoded_with(|buffer| params.exec_input().encode_to_slice(buffer))
        } else {
            &mut []
        };
        let output = &mut scope.take_rest();
        let flags = params.call_flags();

        #[allow(deprecated)] // todo
        let call_result = ext::call(
            *flags,
            enc_callee,
            ref_time_limit,
            proof_size_limit,
            // TODO: cleanup comment?
            &enc_storage_limit,
            enc_transferred_value,
            enc_input,
            Some(output),
        );
        match call_result {
            Ok(()) | Err(ReturnErrorCode::CalleeReverted) => R::decode_output(output),
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    fn invoke_contract_delegate<E, Args, R, Abi>(
        &mut self,
        params: &CallParams<E, DelegateCall, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: AbiEncodeWith<Abi>,
        R: AbiDecodeWith<Abi> + DecodeMessageResult<Abi>,
    {
        let mut scope = self.scoped_buffer();
        let call_flags = params.call_flags();
        let enc_input = if !call_flags.contains(CallFlags::FORWARD_INPUT)
            && !call_flags.contains(CallFlags::CLONE_INPUT)
        {
            scope.take_encoded_with(|buffer| params.exec_input().encode_to_slice(buffer))
        } else {
            &mut []
        };
        let deposit_limit = params.deposit_limit();
        let deposit_limit = remove_option(&mut scope, *deposit_limit);

        let output = &mut scope.take_rest();
        let flags = params.call_flags();
        let enc_address: [u8; 20] = params.address().0;
        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let call_result = ext::delegate_call(
            *flags,
            &enc_address,
            ref_time_limit,
            proof_size_limit,
            &deposit_limit,
            enc_input,
            Some(output),
        );
        match call_result {
            Ok(()) | Err(ReturnErrorCode::CalleeReverted) => R::decode_output(output),
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    #[cfg(feature = "unstable-hostfn")]
    fn instantiate_contract<E, ContractRef, Args, RetType>(
        &mut self,
        params: &CreateParams<E, ContractRef, LimitParamsV2, Args, RetType>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <RetType as ConstructorReturnType<ContractRef>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAddr,
        Args: AbiEncodeWith<Ink>,
        RetType: ConstructorReturnType<ContractRef>,
    {
        let mut scoped = self.scoped_buffer();
        /*
        // todo remove
        let mut foo = [0u8; 15];
        let bar = ink_prelude::format!("params {:?} {:?} {:?}",
                             params.ref_time_limit(),
                             params.proof_size_limit(),
                             params.storage_deposit_limit(),
        );
        foo.copy_from_slice(&bar.as_bytes()[..]);
        self.return_value(
            ReturnFlags::REVERT,
            &foo
        );
        */

        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let storage_deposit_limit = params.storage_deposit_limit().map(|limit| {
            let mut enc_storage_limit = EncodeScope::from(scoped.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        });
        let enc_storage_limit = remove_option(&mut scoped, storage_deposit_limit);

        // todo encodings here are mostly unnecessary, as the type is already 32 bytes
        let enc_code_hash: &mut [u8; 32] = scoped
            .take_encoded(params.code_hash())
            .try_into()
            .expect("unable to take 32 for code_hash");
        let mut enc_endowment = EncodeScope::from(scoped.take(32));
        scale::Encode::encode_to(&params.endowment(), &mut enc_endowment);
        let enc_endowment: &mut [u8; 32] =
            enc_endowment.into_buffer().try_into().unwrap();
        let enc_input = scoped
            .take_encoded_with(|buffer| params.exec_input().encode_to_slice(buffer));
        let mut out_address: [u8; 20] =
            scoped.take(20).try_into().expect("unable to take 20");
        let salt = params.salt_bytes().as_ref();

        let input_and_code_hash = scoped.take(32 + enc_input.len());
        input_and_code_hash[..32].copy_from_slice(enc_code_hash);
        input_and_code_hash[32..].copy_from_slice(enc_input);

        let mut output_data = &mut scoped.take_rest();

        let instantiate_result = ext::instantiate(
            ref_time_limit,
            proof_size_limit,
            &enc_storage_limit,
            enc_endowment,
            input_and_code_hash,
            Some(&mut out_address),
            Some(&mut output_data),
            salt,
        );

        crate::engine::decode_instantiate_result::<_, ContractRef, RetType>(
            instantiate_result.map_err(Into::into),
            &mut &out_address[..],
            &mut &output_data[..],
        )
    }

    #[cfg(feature = "unstable-hostfn")]
    fn terminate_contract(&mut self, beneficiary: Address) -> ! {
        let buffer: &mut [u8; 20] = self.scoped_buffer().take_encoded(&beneficiary)
            [0..20]
            .as_mut()
            .try_into()
            .unwrap();
        ext::terminate(buffer);
    }

    fn transfer<E>(&mut self, destination: Address, value: U256) -> Result<()>
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_callee: &[u8; 20] = destination.as_ref().try_into().unwrap();
        let mut enc_value = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&value, &mut enc_value);
        let enc_value: &mut [u8; 32] = enc_value.into_buffer().try_into().unwrap();

        let mut enc_limit = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&U256::MAX, &mut enc_limit);
        let enc_limit: &mut [u8; 32] = enc_limit.into_buffer().try_into().unwrap();

        let output = &mut scope.take_rest();
        #[allow(deprecated)]
        let call_result = ext::call(
            CallFlags::empty(),
            enc_callee,
            u64::MAX,
            u64::MAX,
            enc_limit,
            enc_value,
            &[],
            Some(output),
        );
        match call_result {
            Ok(()) => {
                // TODO: clean comments?
                // no need to decode, is ()
                Ok(())
            }
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    fn weight_to_fee<E: Environment>(&mut self, gas: u64) -> E::Balance {
        let mut scope = self.scoped_buffer();
        let u256: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        // TODO: needs ref and proof
        ext::weight_to_fee(gas, gas, u256);
        let mut result = <E::Balance as FromLittleEndian>::Bytes::default();
        let len = result.as_ref().len();
        result.as_mut().copy_from_slice(&u256[..len]);
        <E::Balance as FromLittleEndian>::from_le_bytes(result)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn is_contract(&mut self, addr: &Address) -> bool {
        let mut scope = self.scoped_buffer();
        let enc_addr: &mut [u8; 20] =
            scope.take_encoded(addr)[..20].as_mut().try_into().unwrap();
        ext::is_contract(enc_addr)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment,
    {
        ext::caller_is_origin()
    }

    #[cfg(feature = "unstable-hostfn")]
    fn caller_is_root<E>(&mut self) -> bool
    where
        E: Environment,
    {
        ext::caller_is_root()
    }

    fn code_hash(&mut self, addr: &Address) -> Result<H256> {
        let mut scope = self.scoped_buffer();
        // todo can be simplified
        let enc_addr: &mut [u8; 20] =
            scope.take_encoded(addr)[..20].as_mut().try_into().unwrap();
        let output: &mut [u8; 32] =
            scope.take_max_encoded_len::<H256>().try_into().unwrap();
        ext::code_hash(enc_addr, output);
        let hash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn own_code_hash(&mut self) -> Result<H256> {
        let output: &mut [u8; 32] = &mut self
            .scoped_buffer()
            .take_max_encoded_len::<H256>()
            .try_into()
            .unwrap();
        ext::own_code_hash(output);
        let hash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn call_runtime<E, Call>(&mut self, call: &Call) -> Result<()>
    where
        E: Environment,
        Call: scale::Encode,
    {
        let mut scope = self.scoped_buffer();
        let enc_call = scope.take_encoded(call);
        ext::call_runtime(enc_call).map_err(Into::into)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn xcm_execute<E, Call>(&mut self, msg: &VersionedXcm<Call>) -> Result<()>
    where
        E: Environment,
        Call: scale::Encode,
    {
        let mut scope = self.scoped_buffer();

        let enc_msg = scope.take_encoded(msg);

        #[allow(deprecated)]
        ext::xcm_execute(enc_msg).map_err(Into::into)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn xcm_send<E, Call>(
        &mut self,
        dest: &xcm::VersionedLocation,
        msg: &VersionedXcm<Call>,
    ) -> Result<xcm::v4::XcmHash>
    where
        E: Environment,
        Call: scale::Encode,
    {
        let mut scope = self.scoped_buffer();
        let output = scope.take(32);
        scope.append_encoded(dest);
        let enc_dest = scope.take_appended();

        scope.append_encoded(msg);
        let enc_msg = scope.take_appended();
        #[allow(deprecated)]
        ext::xcm_send(enc_dest, enc_msg, output.try_into().unwrap())?;
        let hash: xcm::v4::XcmHash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }
}

// todo make this const
fn to_u256(scope: &mut ScopedBuffer, value: Option<U256>) -> [u8; 32] {
    let limit = match value {
        None => U256::MAX,
        Some(u256) => u256,
    };
    let mut enc_storage_limit = EncodeScope::from(scope.take(32));
    scale::Encode::encode_to(&limit, &mut enc_storage_limit);
    let enc_storage_limit: [u8; 32] = enc_storage_limit.into_buffer().try_into().unwrap();
    enc_storage_limit
}

fn remove_option(scope: &mut ScopedBuffer, opt: Option<[u8; 32]>) -> [u8; 32] {
    match opt {
        None => {
            let limit = U256::MAX; // corresponds to no deposit limit, defined in `pallet-revive`
            let mut enc_storage_limit = EncodeScope::from(scope.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        }
        Some(bytes) => bytes,
    }
}
