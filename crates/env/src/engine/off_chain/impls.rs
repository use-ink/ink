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

use super::EnvInstance;
use crate::{
    call::{
        Call,
        CallParams,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAddr,
        LimitParamsV2,
    },
    event::{
        Event,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    DecodeDispatch,
    DispatchError,
    EnvBackend,
    Result,
    TypedEnvBackend,
};
use ink_primitives::{
    types::Environment,
    H160,
    H256,
    U256,
};
use ink_storage_traits::{
    Storable,
};
use pallet_revive_uapi::{
    ReturnFlags,
};

impl EnvBackend for EnvInstance {
    fn set_contract_storage<K, V>(&mut self, _key: &K, _value: &V) -> Option<u32>
    where
        K: scale::Encode,
        V: Storable,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn get_contract_storage<K, R>(&mut self, _key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn take_contract_storage<K, R>(&mut self, _key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn contains_contract_storage<K>(&mut self, _key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn clear_contract_storage<K>(&mut self,_key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn decode_input<T>(&mut self) -> core::result::Result<T, DispatchError>
    where
        T: DecodeDispatch,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn return_value<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: scale::Encode,
    {
        panic!("enable feature `std` to use `return_value()`")
    }

    fn return_value_rlp<R>(&mut self, _flags: ReturnFlags, _return_value: &R) -> !
    where
        R: alloy_rlp::Encodable,
    {
        unimplemented!("the off-chain env does not implement `return_value_rlp`")
    }

    fn debug_message(&mut self, _message: &str) {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn hash_bytes<H>(&mut self, _input: &[u8], _output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn hash_encoded<H, T>(&mut self, _input: &T, _output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
        T: scale::Encode,
    {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn ecdsa_recover(
        &mut self,
        _signature: &[u8; 65],
        _message_hash: &[u8; 32],
        _output: &mut [u8; 33],
    ) -> Result<()> {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn ecdsa_to_eth_address(
        &mut self,
        _pubkey: &[u8; 33],
        _output: &mut [u8; 20],
    ) -> Result<()> {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn sr25519_verify(
        &mut self,
        _signature: &[u8; 64],
        _message: &[u8],
        _pub_key: &[u8; 32],
    ) -> Result<()> {
        unimplemented!("the off-chain env does not implement `input`")
    }

    fn call_chain_extension<I, T, E, ErrorCode, F, D>(
        &mut self,
        _id: u32,
        _input: &I,
        _status_to_result: F,
        _decode_to_result: D,
    ) -> ::core::result::Result<T, E>
    where
        I: scale::Encode,
        T: scale::Decode,
        E: From<ErrorCode>,
        F: FnOnce(u32) -> ::core::result::Result<(), ErrorCode>,
        D: FnOnce(&[u8]) -> ::core::result::Result<T, E>,
    {
        unimplemented!("foo");
    }

    fn set_code_hash(&mut self, _code_hash: &H256) -> Result<()> {
        unimplemented!("foo");
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller(&mut self) -> H160 {
        unimplemented!("foo");
    }

    fn transferred_value(&mut self) -> U256 {
        unimplemented!("foo");
    }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        unimplemented!("foo");
    }

    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        unimplemented!("foo");
    }

    fn address(&mut self) -> H160 {
        unimplemented!("foo");
    }

    fn balance(&mut self) -> U256 {
        unimplemented!("foo");
    }

    fn block_number<E: Environment>(&mut self) -> E::BlockNumber {
        unimplemented!("foo");
    }

    fn minimum_balance<E: Environment>(&mut self) -> E::Balance {
        unimplemented!("foo");
    }

    fn emit_event<E, Evt>(&mut self, _event: Evt)
    where
        E: Environment,
        Evt: Event,
    {
        unimplemented!("foo");
    }

    fn invoke_contract<E, Args, R>(
        &mut self,
        _params: &CallParams<E, Call, Args, R>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        unimplemented!("foo");
    }

    fn invoke_contract_delegate<E, Args, R>(
        &mut self,
        _params: &CallParams<E, DelegateCall, Args, R>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        unimplemented!("foo");
    }

    fn instantiate_contract<E, ContractRef, Args, R>(
        &mut self,
        _params: &CreateParams<E, ContractRef, LimitParamsV2, Args, R>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <R as ConstructorReturnType<ContractRef>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAddr + crate::ContractReverseReference,
        <ContractRef as crate::ContractReverseReference>::Type:
            crate::reflect::ContractConstructorDecoder,
        Args: scale::Encode,
        R: ConstructorReturnType<ContractRef>,
    {
        unimplemented!("foo");
    }

    fn terminate_contract(&mut self, _beneficiary: H160) -> ! {
        unimplemented!("foo");
    }

    fn transfer<E>(&mut self, _destination: H160, _value: U256) -> Result<()>
    where
        E: Environment,
    {
        unimplemented!("foo");
    }

    fn weight_to_fee<E: Environment>(&mut self, _gas: u64) -> E::Balance {
        unimplemented!("foo");
    }

    fn is_contract(&mut self, _account: &H160) -> bool {
        unimplemented!("foo");
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

    fn code_hash(&mut self, _addr: &H160) -> Result<H256> {
        unimplemented!("foo");
    }

    fn own_code_hash(&mut self) -> Result<H256> {
        unimplemented!("foo");
    }

    fn call_runtime<E, Call>(&mut self, _call: &Call) -> Result<()>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `call_runtime`")
    }

    fn lock_delegate_dependency<E>(&mut self, _code_hash: &H256)
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support delegate dependencies")
    }

    fn xcm_execute<E, Call>(&mut self, _msg: &xcm::VersionedXcm<Call>) -> Result<()>
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support `xcm_execute`")
    }

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

    fn unlock_delegate_dependency<E>(&mut self, _code_hash: &H256)
    where
        E: Environment,
    {
        unimplemented!("off-chain environment does not support delegate dependencies")
    }
}
