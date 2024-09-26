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

use crate::{
    call::{
        common::{
            ReturnType,
            Set,
            Unset,
        },
        execution::EmptyArgumentList,
        CallBuilder,
        CallParams,
        ExecutionInput,
    },
    Environment,
    Error,
};
use pallet_revive_uapi::CallFlags;

/// The `delegatecall` call type. Performs a call with the given code hash.
#[derive(Clone)]
pub struct DelegateCall<E: Environment> {
    code_hash: E::Hash,
    call_flags: CallFlags,
}

impl<E: Environment> DelegateCall<E> {
    /// Returns a clean builder for [`DelegateCall`]
    pub const fn new(code_hash: E::Hash) -> Self {
        DelegateCall {
            code_hash,
            call_flags: CallFlags::empty(),
        }
    }

    /// Sets the `code_hash` to perform a delegate call with.
    pub fn code_hash(self, code_hash: E::Hash) -> Self {
        DelegateCall {
            code_hash,
            call_flags: CallFlags::empty(),
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<DelegateCall<E>>, Args, RetType>
where
    E: Environment,
{
    /// Sets the `code_hash` to perform a delegate call with.
    pub fn code_hash(self, code_hash: E::Hash) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(DelegateCall {
                code_hash,
                ..call_type
            }),
            ..self
        }
    }

    /// Sets the `code_hash` to perform a delegate call with.
    pub fn call_flags(self, call_flags: CallFlags) -> Self {
        CallBuilder {
            call_type: Set(DelegateCall {
                call_flags,
                ..self.call_type.value()
            }),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, Args, RetType>
    CallBuilder<
        E,
        Set<DelegateCall<E>>,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<RetType>>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, DelegateCall<E>, Args, RetType> {
        CallParams {
            call_type: self.call_type.value(),
            _return_type: Default::default(),
            exec_input: self.exec_input.value(),
            _phantom: self._phantom,
        }
    }
}

impl<E, RetType>
    CallBuilder<
        E,
        Set<DelegateCall<E>>,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Unset<RetType>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, DelegateCall<E>, EmptyArgumentList, ()> {
        CallParams {
            call_type: self.call_type.value(),
            _return_type: Default::default(),
            exec_input: Default::default(),
            _phantom: self._phantom,
        }
    }
}

impl<E>
    CallBuilder<
        E,
        Set<DelegateCall<E>>,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Unset<ReturnType<()>>,
    >
where
    E: Environment,
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
    /// On failure this an [`ink::env::Error`][`crate::Error`] which can be handled by the
    /// caller.
    pub fn try_invoke(self) -> Result<ink_primitives::MessageResult<()>, Error> {
        self.params().try_invoke()
    }
}

impl<E, Args, R>
    CallBuilder<E, Set<DelegateCall<E>>, Set<ExecutionInput<Args>>, Set<ReturnType<R>>>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
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

impl<E, Args, R> CallParams<E, DelegateCall<E>, Args, R>
where
    E: Environment,
{
    /// Returns the code hash which we use to perform a delegate call.
    #[inline]
    pub fn code_hash(&self) -> &E::Hash {
        &self.call_type.code_hash
    }

    /// Returns the call flags.
    #[inline]
    pub fn call_flags(&self) -> &CallFlags {
        &self.call_type.call_flags
    }
}

impl<E, Args, R> CallParams<E, DelegateCall<E>, Args, R>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
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
