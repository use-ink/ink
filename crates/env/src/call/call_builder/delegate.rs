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

use ink_primitives::{
    Address,
    abi::AbiEncodeWith,
};
use pallet_revive_uapi::CallFlags;

use crate::{
    Error,
    call::{
        CallBuilder,
        CallParams,
        ExecutionInput,
        common::{
            ReturnType,
            Set,
            Unset,
        },
        execution::EmptyArgumentList,
        utils::DecodeMessageResult,
    },
    types::Environment,
};

/// The `delegatecall` call type. Performs a call with the given code hash.
#[derive(Clone)]
pub struct DelegateCall {
    // todo comments please
    address: Address,
    flags: CallFlags,
    ref_time_limit: u64,
    proof_size_limit: u64,
    // todo U256
    deposit_limit: Option<[u8; 32]>,
}

impl DelegateCall {
    /// Returns a clean builder for [`DelegateCall`]
    pub const fn new(address: Address) -> Self {
        DelegateCall {
            address,
            flags: CallFlags::empty(),
            ref_time_limit: u64::MAX,
            proof_size_limit: u64::MAX,
            deposit_limit: None,
        }
    }

    /// Sets the `address` to perform a delegate call with.
    pub fn address(self, address: Address) -> Self {
        DelegateCall {
            address,
            flags: CallFlags::empty(),
            ref_time_limit: u64::MAX,
            proof_size_limit: u64::MAX,
            deposit_limit: None,
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<DelegateCall>, Args, RetType>
where
    E: Environment,
{
    /// Sets the `address` to perform a delegate call with.
    pub fn address(self, address: Address) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(DelegateCall {
                address,
                ..call_type
            }),
            ..self
        }
    }

    /// Sets the `CallFlags` to perform a delegate call with.
    pub fn call_flags(self, call_flags: CallFlags) -> Self {
        CallBuilder {
            call_type: Set(DelegateCall {
                flags: call_flags,
                ..self.call_type.value()
            }),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, Args, RetType, Abi>
    CallBuilder<
        E,
        Set<DelegateCall>,
        Set<ExecutionInput<Args, Abi>>,
        Set<ReturnType<RetType>>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, DelegateCall, Args, RetType, Abi> {
        CallParams {
            call_type: self.call_type.value(),
            _return_type: Default::default(),
            exec_input: self.exec_input.value(),
            _phantom: self._phantom,
        }
    }
}

impl<E, RetType, Abi>
    CallBuilder<
        E,
        Set<DelegateCall>,
        Unset<ExecutionInput<EmptyArgumentList<Abi>, Abi>>,
        Unset<RetType>,
    >
where
    E: Environment,
    EmptyArgumentList<Abi>: AbiEncodeWith<Abi>,
    (): DecodeMessageResult<Abi>,
    Abi: Default,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, DelegateCall, EmptyArgumentList<Abi>, (), Abi> {
        CallParams {
            call_type: self.call_type.value(),
            _return_type: Default::default(),
            exec_input: Default::default(),
            _phantom: self._phantom,
        }
    }
}

impl<E, Abi>
    CallBuilder<
        E,
        Set<DelegateCall>,
        Unset<ExecutionInput<EmptyArgumentList<Abi>, Abi>>,
        Unset<ReturnType<()>>,
    >
where
    E: Environment,
    EmptyArgumentList<Abi>: AbiEncodeWith<Abi>,
    (): DecodeMessageResult<Abi>,
    Abi: Default,
{
    /// Invokes the cross-chain function call using Delegate Call semantics.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`]
    /// If you want to handle those use the [`try_invoke`][`CallBuilder::try_invoke`]
    /// method instead.
    pub fn invoke(self) {
        self.params().invoke()
    }

    /// Invokes the cross-chain function call using Delegate Call semantics.
    ///
    /// # Note
    ///
    /// On failure this returns an [`ink::env::Error`][`crate::Error`] which can be
    /// handled by the caller.
    pub fn try_invoke(self) -> Result<ink_primitives::MessageResult<()>, Error> {
        self.params().try_invoke()
    }
}

impl<E, Args, R, Abi>
    CallBuilder<E, Set<DelegateCall>, Set<ExecutionInput<Args, Abi>>, Set<ReturnType<R>>>
where
    E: Environment,
    Args: AbiEncodeWith<Abi>,
    R: DecodeMessageResult<Abi>,
{
    /// Invokes the cross-chain function call using Delegate Call semantics and returns
    /// the result.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_invoke`][`CallBuilder::try_invoke`] method instead.
    pub fn invoke(self) -> R {
        self.params().invoke()
    }

    /// Invokes the cross-chain function call using Delegate Call semantics and returns
    /// the result.
    ///
    /// # Note
    ///
    /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
    /// handled by the caller.
    pub fn try_invoke(self) -> Result<ink_primitives::MessageResult<R>, Error> {
        self.params().try_invoke()
    }
}

impl<E, Args, R, Abi> CallParams<E, DelegateCall, Args, R, Abi>
where
    E: Environment,
{
    /// Returns the call flags.
    #[inline]
    pub fn call_flags(&self) -> &CallFlags {
        &self.call_type.flags
    }

    /// Returns the contract address which we use to perform a delegate call.
    #[inline]
    pub fn address(&self) -> &Address {
        &self.call_type.address
    }

    /// Returns the `ref_time_limit` which we use to perform a delegate call.
    #[inline]
    pub fn ref_time_limit(&self) -> u64 {
        self.call_type.ref_time_limit
    }

    /// Returns the `proof_size_limit` which we use to perform a delegate call.
    #[inline]
    pub fn proof_size_limit(&self) -> u64 {
        self.call_type.proof_size_limit
    }

    /// Returns the `deposit_limit` which we use to perform a delegate call.
    #[inline]
    pub fn deposit_limit(&self) -> &Option<[u8; 32]> {
        &self.call_type.deposit_limit
    }
}

impl<E, Args, R, Abi> CallParams<E, DelegateCall, Args, R, Abi>
where
    E: Environment,
    Args: AbiEncodeWith<Abi>,
    R: DecodeMessageResult<Abi>,
{
    /// Invoke the contract using Delegate Call semantics with the given built-up call
    /// parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_invoke`][`CallParams::try_invoke`] method instead.
    pub fn invoke(&self) -> R {
        crate::invoke_contract_delegate(self)
            .unwrap_or_else(|env_error| {
                panic!("Cross-contract call failed with {env_error:?}")
            })
            .unwrap_or_else(|lang_error| {
                panic!("Cross-contract call failed with {lang_error:?}")
            })
    }

    /// Invoke the contract using Delegate Call semantics with the given built-up call
    /// parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Note
    ///
    /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
    /// handled by the caller.
    pub fn try_invoke(&self) -> Result<ink_primitives::MessageResult<R>, crate::Error> {
        crate::invoke_contract_delegate(self)
    }
}
