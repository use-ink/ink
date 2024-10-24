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

use crate::engine::on_chain::{
    EncodeScope,
    EnvInstance,
    ScopedBuffer,
};
use crate::{
    call::{
        Call,
        CallParams,
        // CallV1,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAccountId,
        LimitParamsV1,
        LimitParamsV2,
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
    Clear,
    EnvBackend,
    Environment,
    FromLittleEndian,
    Result,
    TypedEnvBackend,
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
use xcm::VersionedXcm;

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

    fn expect(&mut self, expected_topics: usize) {
        self.scoped_buffer
            .append_encoded(&scale::Compact(expected_topics as u32));
    }

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
        let u256: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        ext_fn(u256);
        let mut result = <T as FromLittleEndian>::Bytes::default();
        let len = result.as_ref().len();
        result.as_mut()[..].copy_from_slice(&u256[..len]);
        <T as FromLittleEndian>::from_le_bytes(result)
    }

    /// Returns the contract property value.
    #[inline(always)]
    fn get_property<T>(&mut self, ext_fn: fn(output: &mut &mut [u8])) -> Result<T>
    where
        T: scale::Decode,
    {
        let full_scope = &mut self.scoped_buffer().take_rest();
        ext_fn(full_scope);
        scale::Decode::decode(&mut &full_scope[..]).map_err(Into::into)
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

    fn contains_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        ext::contains_storage(STORAGE_FLAGS, key)
    }

    fn clear_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        ext::clear_storage(STORAGE_FLAGS, key)
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
        let mut scope = EncodeScope::from(&mut self.buffer[..]);
        return_value.encode_to(&mut scope);
        let len = scope.len();
        ext::return_value(flags, &self.buffer[..][..len]);
    }

    #[cfg(not(feature = "ink-debug"))]
    /// A no-op. Enable the `ink-debug` feature for debug messages.
    fn debug_message(&mut self, _content: &str) {}

    #[cfg(feature = "ink-debug")]
    fn debug_message(&mut self, content: &str) {
        static mut DEBUG_ENABLED: bool = false;
        static mut FIRST_RUN: bool = true;

        // SAFETY: safe because executing in a single threaded context
        // We need those two variables in order to make sure that the assignment is
        // performed in the "logging enabled" case. This is because during RPC
        // execution logging might be enabled while it is disabled during the
        // actual execution as part of a transaction. The gas estimation takes
        // place during RPC execution. We want to overestimate instead
        // of underestimate gas usage. Otherwise using this estimate could lead to a out
        // of gas error.
        if unsafe { DEBUG_ENABLED || FIRST_RUN } {
            let ret_code = ext::debug_message(content.as_bytes());
            if !matches!(ret_code, Err(ReturnErrorCode::LoggingDisabled)) {
                // SAFETY: safe because executing in a single threaded context
                unsafe { DEBUG_ENABLED = true }
            }
            // SAFETY: safe because executing in a single threaded context
            unsafe { FIRST_RUN = false }
        }
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
        ext::ecdsa_recover(signature, message_hash, output).map_err(Into::into)
    }

    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
    ) -> Result<()> {
        ext::ecdsa_to_eth_address(pubkey, output).map_err(Into::into)
    }

    fn sr25519_verify(
        &mut self,
        signature: &[u8; 64],
        message: &[u8],
        pub_key: &[u8; 32],
    ) -> Result<()> {
        ext::sr25519_verify(signature, message, pub_key).map_err(Into::into)
    }

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

    fn set_code_hash(&mut self, code_hash_ptr: &[u8]) -> Result<()> {
        let code_hash: &[u8; 32] = code_hash_ptr.try_into().unwrap();
        ext::set_code_hash(code_hash).map_err(Into::into)
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller<E: Environment>(&mut self) -> E::AccountId {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::caller(h160);

        let account_id: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        ext::to_account_id(h160, account_id);

        scale::Decode::decode(&mut &account_id[..])
            .expect("The executed contract must have a caller with a valid account id.")
    }

    fn transferred_value<E: Environment>(&mut self) -> E::Balance {
        self.get_property_little_endian::<E::Balance>(ext::value_transferred)
    }

    // fn gas_left<E: Environment>(&mut self) -> u64 {
    //     self.get_property_little_endian::<u64>(ext::gas_left)
    // }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        self.get_property_little_endian::<E::Timestamp>(ext::now)
    }

    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        let mut scope = self.scoped_buffer();

        let account_id: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        account_id[20..].fill(0xEE);
        let h160: &mut [u8; 20] = account_id[..20].as_mut().try_into().unwrap();
        ext::address(h160);

        scale::Decode::decode(&mut &account_id[..])
            .expect("A contract being executed must have a valid account id.")
    }

    fn balance<E: Environment>(&mut self) -> E::Balance {
        self.get_property_little_endian::<E::Balance>(ext::balance)
    }

    fn block_number<E: Environment>(&mut self) -> E::BlockNumber {
        self.get_property_little_endian::<E::BlockNumber>(ext::block_number)
    }

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
        let enc_topics = &enc_topics
            .chunks_exact(32)
            .map(|c| c.try_into().unwrap())
            .collect::<ink_prelude::vec::Vec<_>>();
        let enc_data = scope.take_encoded(&event);
        ext::deposit_event(enc_topics, enc_data);
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
        let mut scope = self.scoped_buffer();
        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let storage_deposit_limit = params.storage_deposit_limit().map(|limit| {
            let mut enc_storage_limit = EncodeScope::from(scope.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: &mut [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        });
        let enc_callee: &[u8; 20] = params.callee().as_ref().try_into().unwrap();
        let mut enc_transferred_value = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&params.transferred_value(), &mut enc_transferred_value);
        let enc_transferred_value: &mut [u8; 32] =
            enc_transferred_value.into_buffer().try_into().unwrap();
        let call_flags = params.call_flags();
        let enc_input = if !call_flags.contains(CallFlags::FORWARD_INPUT)
            && !call_flags.contains(CallFlags::CLONE_INPUT)
        {
            scope.take_encoded(params.exec_input())
        } else {
            &mut []
        };
        let output = &mut scope.take_rest();
        let flags = params.call_flags();
        #[allow(deprecated)]
        let call_result = ext::call(
            *flags,
            enc_callee,
            ref_time_limit,
            proof_size_limit,
            storage_deposit_limit.as_deref(),
            enc_transferred_value,
            enc_input,
            Some(output),
        );
        match call_result {
            Ok(()) | Err(ReturnErrorCode::CalleeReverted) => {
                let decoded = scale::DecodeAll::decode_all(&mut &output[..])?;
                Ok(decoded)
            }
            Err(actual_error) => Err(actual_error.into()),
        }
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
        let mut scope = self.scoped_buffer();
        let call_flags = params.call_flags();
        let enc_code_hash: &mut [u8; 32] =
            scope.take_encoded(params.code_hash()).try_into().unwrap();
        let enc_input = if !call_flags.contains(CallFlags::FORWARD_INPUT)
            && !call_flags.contains(CallFlags::CLONE_INPUT)
        {
            scope.take_encoded(params.exec_input())
        } else {
            &mut []
        };
        let output = &mut scope.take_rest();
        let flags = params.call_flags();
        let call_result =
            ext::delegate_call(*flags, enc_code_hash, enc_input, Some(output));
        match call_result {
            Ok(()) | Err(ReturnErrorCode::CalleeReverted) => {
                let decoded = scale::DecodeAll::decode_all(&mut &output[..])?;
                Ok(decoded)
            }
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    fn instantiate_contract<E, ContractRef, Args, Salt, RetType>(
        &mut self,
        params: &CreateParams<E, ContractRef, LimitParamsV2<E>, Args, Salt, RetType>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <RetType as ConstructorReturnType<ContractRef>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAccountId<E>,
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
        RetType: ConstructorReturnType<ContractRef>,
    {
        let mut scoped = self.scoped_buffer();
        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let storage_deposit_limit = params.storage_deposit_limit().map(|limit| {
            let mut enc_storage_limit = EncodeScope::from(scoped.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: &mut [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        });
        let enc_code_hash: &mut [u8; 32] =
            scoped.take_encoded(params.code_hash()).try_into().unwrap();
        let mut enc_endowment = EncodeScope::from(scoped.take(32));
        scale::Encode::encode_to(&params.endowment(), &mut enc_endowment);
        let enc_endowment: &mut [u8; 32] =
            enc_endowment.into_buffer().try_into().unwrap();
        let enc_input = scoped.take_encoded(params.exec_input());
        let out_address: &mut [u8; 20] = scoped.take(20).try_into().unwrap();
        let salt: &[u8; 32] = params.salt_bytes().as_ref().try_into().unwrap();
        let out_return_value = &mut scoped.take_rest();

        let instantiate_result = ext::instantiate(
            enc_code_hash,
            ref_time_limit,
            proof_size_limit,
            storage_deposit_limit.as_deref(),
            enc_endowment,
            enc_input,
            Some(out_address),
            Some(out_return_value),
            Some(salt),
        );

        crate::engine::decode_instantiate_result::<_, E, ContractRef, RetType>(
            instantiate_result.map_err(Into::into),
            &mut &out_address[..],
            &mut &out_return_value[..],
        )
    }

    fn terminate_contract<E>(&mut self, beneficiary: E::AccountId) -> !
    where
        E: Environment,
    {
        let buffer: &mut [u8; 20] = self
            .scoped_buffer()
            .take_encoded(&beneficiary)[0..20].as_mut()
            .try_into()
            .unwrap();
        ext::terminate(buffer);
    }

    fn transfer<E>(&mut self, destination: E::AccountId, value: E::Balance) -> Result<()>
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_destination: &mut [u8; 20] = scope.take_encoded(&destination)[..20].as_mut().try_into().unwrap();
        let enc_value = scope.take(32);
        let mut encode_scope = EncodeScope::from(enc_value);
        scale::Encode::encode_to(&value, &mut encode_scope);
        let enc_value: &mut [u8; 32] = array_mut_ref!(encode_scope.into_buffer(), 0, 32);
        ext::transfer(enc_destination, enc_value).map_err(Into::into)
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

    fn is_contract<E>(&mut self, account_id: &E::AccountId) -> bool
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_account_id: &mut [u8; 20] =
            scope.take_encoded(account_id)[..20].as_mut().try_into().unwrap();
        ext::is_contract(enc_account_id)
    }

    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment,
    {
        ext::caller_is_origin()
    }

    fn code_hash<E>(&mut self, account_id: &E::AccountId) -> Result<E::Hash>
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_account_id: &mut [u8; 20] = scope.take_encoded(account_id)[..20].as_mut().try_into().unwrap();
        let output: &mut [u8; 32] =
            scope.take_max_encoded_len::<E::Hash>().try_into().unwrap();
        ext::code_hash(enc_account_id, output)?;
        let hash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }

    fn own_code_hash<E>(&mut self) -> Result<E::Hash>
    where
        E: Environment,
    {
        let output: &mut [u8; 32] = &mut self
            .scoped_buffer()
            .take_max_encoded_len::<E::Hash>()
            .try_into()
            .unwrap();
        ext::own_code_hash(output);
        let hash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }

    fn call_runtime<E, Call>(&mut self, call: &Call) -> Result<()>
    where
        E: Environment,
        Call: scale::Encode,
    {
        let mut scope = self.scoped_buffer();
        let enc_call = scope.take_encoded(call);
        ext::call_runtime(enc_call).map_err(Into::into)
    }

    fn lock_delegate_dependency<E>(&mut self, code_hash: &E::Hash)
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_code_hash: &mut [u8; 32] =
            scope.take_encoded(code_hash).try_into().unwrap();
        ext::lock_delegate_dependency(enc_code_hash)
    }

    fn unlock_delegate_dependency<E>(&mut self, code_hash: &E::Hash)
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_code_hash: &mut [u8; 32] =
            scope.take_encoded(code_hash).try_into().unwrap();
        ext::unlock_delegate_dependency(enc_code_hash)
    }

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
