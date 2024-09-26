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

use super::CallParams;
use crate::{
    call::{
        common::{
            ReturnType,
            Set,
            Unset,
        },
        execution::EmptyArgumentList,
        CallBuilder,
        ExecutionInput,
    },
    Environment,
    Error,
    Gas,
};
use num_traits::Zero;
use pallet_revive_uapi::CallFlags;

/// The legacy call type for cross-contract calls. Performs a cross-contract call to
/// `callee` with gas limit `gas_limit`, transferring `transferred_value` of currency.
///
/// Calls into the original `call` host function.
#[derive(Clone)]
pub struct CallV1<E: Environment> {
    pub(crate) callee: E::AccountId,
    pub(crate) gas_limit: Gas,
    pub(crate) transferred_value: E::Balance,
    pub(crate) call_flags: CallFlags,
}

impl<E: Environment> CallV1<E> {
    /// Returns a clean builder for [`CallV1`].
    pub fn new(callee: E::AccountId) -> Self {
        Self {
            callee,
            gas_limit: Default::default(),
            transferred_value: E::Balance::zero(),
            call_flags: CallFlags::empty(),
        }
    }
}

impl<E> CallV1<E>
where
    E: Environment,
{
    /// Sets the `gas_limit` for the current cross-contract call.
    pub fn gas_limit(self, gas_limit: Gas) -> Self {
        CallV1 { gas_limit, ..self }
    }

    /// Sets the `transferred_value` for the current cross-contract call.
    pub fn transferred_value(self, transferred_value: E::Balance) -> Self {
        CallV1 {
            transferred_value,
            ..self
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<CallV1<E>>, Args, RetType>
where
    E: Environment,
{
    /// Sets the `gas_limit` for the current cross-contract call.
    pub fn gas_limit(self, gas_limit: Gas) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(CallV1 {
                callee: call_type.callee,
                gas_limit,
                transferred_value: call_type.transferred_value,
                call_flags: call_type.call_flags,
            }),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }

    /// Sets the `transferred_value` for the current cross-contract call.
    pub fn transferred_value(self, transferred_value: E::Balance) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(CallV1 {
                callee: call_type.callee,
                gas_limit: call_type.gas_limit,
                transferred_value,
                call_flags: call_type.call_flags,
            }),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }

    /// Sets the flags used to change the behavior of the contract call.
    #[inline]
    #[must_use]
    pub fn call_flags(self, call_flags: CallFlags) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(CallV1 {
                callee: call_type.callee,
                gas_limit: call_type.gas_limit,
                transferred_value: call_type.transferred_value,
                call_flags,
            }),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, Args, RetType>
    CallBuilder<E, Set<CallV1<E>>, Set<ExecutionInput<Args>>, Set<ReturnType<RetType>>>
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, CallV1<E>, Args, RetType> {
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
        Set<CallV1<E>>,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Unset<RetType>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, CallV1<E>, EmptyArgumentList, ()> {
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
        Set<CallV1<E>>,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Unset<ReturnType<()>>,
    >
where
    E: Environment,
{
    /// Invokes the cross-chain function call.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_invoke`][`CallBuilder::try_invoke`] method instead.
    pub fn invoke(self) {
        self.params().invoke()
    }

    /// Invokes the cross-chain function call.
    ///
    /// # Note
    ///
    /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
    /// handled by the caller.
    pub fn try_invoke(self) -> Result<ink_primitives::MessageResult<()>, Error> {
        self.params().try_invoke()
    }
}

impl<E, Args, R>
    CallBuilder<E, Set<CallV1<E>>, Set<ExecutionInput<Args>>, Set<ReturnType<R>>>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Invokes the cross-chain function call and returns the result.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_invoke`][`CallBuilder::try_invoke`] method instead.
    pub fn invoke(self) -> R {
        self.params().invoke()
    }

    /// Invokes the cross-chain function call and returns the result.
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

impl<E, Args, R> CallParams<E, CallV1<E>, Args, R>
where
    E: Environment,
{
    /// Returns the account ID of the called contract instance.
    #[inline]
    pub fn callee(&self) -> &E::AccountId {
        &self.call_type.callee
    }

    /// Returns the chosen gas limit for the called contract execution.
    #[inline]
    pub fn gas_limit(&self) -> Gas {
        self.call_type.gas_limit
    }

    /// Returns the transferred value for the called contract.
    #[inline]
    pub fn transferred_value(&self) -> &E::Balance {
        &self.call_type.transferred_value
    }

    /// Returns the call flags.
    #[inline]
    pub fn call_flags(&self) -> &CallFlags {
        &self.call_type.call_flags
    }
}

impl<E, Args, R> CallParams<E, CallV1<E>, Args, R>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Invokes the contract with the given built-up call parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_invoke`][`CallParams::try_invoke`] method instead.
    pub fn invoke(&self) -> R {
        crate::invoke_contract_v1(self)
            .unwrap_or_else(|env_error| {
                panic!("Cross-contract call failed with {env_error:?}")
            })
            .unwrap_or_else(|lang_error| {
                panic!("Cross-contract call failed with {lang_error:?}")
            })
    }

    /// Invokes the contract with the given built-up call parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Note
    ///
    /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
    /// handled by the caller.
    pub fn try_invoke(&self) -> Result<ink_primitives::MessageResult<R>, crate::Error> {
        crate::invoke_contract_v1(self)
    }
}
