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
        // CallV1,
        ExecutionInput,
    },
    Environment,
    Error,
    Gas,
};
use num_traits::Zero;
use pallet_revive_uapi::CallFlags;

/// The default call type for cross-contract calls, for calling into the latest `call_v2`
/// host function. This adds the additional weight limit parameter `proof_size_limit` as
/// well as `storage_deposit_limit`.
#[derive(Clone)]
pub struct Call<E: Environment> {
    callee: E::AccountId,
    ref_time_limit: u64,
    proof_size_limit: u64,
    storage_deposit_limit: Option<E::Balance>,
    transferred_value: E::Balance,
    call_flags: CallFlags,
}

impl<E: Environment> Call<E> {
    /// Returns a clean builder for [`Call`].
    pub fn new(callee: E::AccountId) -> Self {
        Self {
            callee,
            ref_time_limit: Default::default(),
            proof_size_limit: Default::default(),
            storage_deposit_limit: None,
            transferred_value: E::Balance::zero(),
            call_flags: CallFlags::empty(),
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<Call<E>>, Args, RetType>
where
    E: Environment,
{
    /// Switch to the original `call` host function API, which only allows the `gas_limit`
    /// limit parameter (equivalent to the `ref_time_limit` in the latest `call_v2`).
    ///
    /// This method instance is used to allow usage of the generated call builder methods
    /// for messages which initialize the builder with the new [`Call`] type.
    // pub fn call_v1(self) -> CallBuilder<E, Set<CallV1<E>>, Args, RetType> {
    //     let call_type = self.call_type.value();
    //     CallBuilder {
    //         call_type: Set(CallV1 {
    //             callee: call_type.callee,
    //             gas_limit: call_type.ref_time_limit,
    //             transferred_value: call_type.transferred_value,
    //             call_flags: call_type.call_flags,
    //         }),
    //         exec_input: self.exec_input,
    //         return_type: self.return_type,
    //         _phantom: Default::default(),
    //     }
    // }

    /// Sets the `ref_time_limit` part of the weight limit for the current cross-contract
    /// call.
    ///
    /// `ref_time` refers to the amount of computational time that can be
    /// used for execution, in picoseconds. You can find more info
    /// [here](https://use.ink/basics/gas).
    pub fn ref_time_limit(self, ref_time_limit: Gas) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                ref_time_limit,
                ..call_type
            }),
            ..self
        }
    }

    /// Sets the `proof_size_limit` part of the weight limit for the current
    /// cross-contract call.
    ///
    /// `proof_size` refers to the amount of storage in bytes that a transaction
    /// is allowed to read. You can find more info
    /// [here](https://use.ink/basics/gas).
    ///
    /// **Note**
    ///
    /// This limit is only relevant for parachains, not for standalone chains which do not
    /// require sending a Proof-of-validity to the relay chain.
    pub fn proof_size_limit(self, proof_size_limit: Gas) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                proof_size_limit,
                ..call_type
            }),
            ..self
        }
    }

    /// Sets the `storage_deposit_limit` for the current cross-contract call.
    ///
    /// The `storage_deposit_limit` specifies the amount of user funds that
    /// can be charged for creating storage. You can find more info
    /// [here](https://use.ink/basics/gas).
    pub fn storage_deposit_limit(self, storage_deposit_limit: E::Balance) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                storage_deposit_limit: Some(storage_deposit_limit),
                ..call_type
            }),
            ..self
        }
    }

    /// Sets the `transferred_value` for the current cross-contract call.
    ///
    /// This value specifies the amount of user funds that are transferred
    /// to the other contract with this call.
    pub fn transferred_value(self, transferred_value: E::Balance) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                transferred_value,
                ..call_type
            }),
            ..self
        }
    }

    /// Sets the `call_flags` for the current cross-contract call.
    ///
    /// These flags are used to change the behavior of the contract call.
    pub fn call_flags(self, call_flags: CallFlags) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                call_flags,
                ..call_type
            }),
            ..self
        }
    }
}

impl<E, Args, RetType>
    CallBuilder<E, Set<Call<E>>, Set<ExecutionInput<Args>>, Set<ReturnType<RetType>>>
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, Call<E>, Args, RetType> {
        CallParams {
            call_type: self.call_type.value(),
            _return_type: Default::default(),
            exec_input: self.exec_input.value(),
            _phantom: self._phantom,
        }
    }
}

impl<E, RetType>
    CallBuilder<E, Set<Call<E>>, Unset<ExecutionInput<EmptyArgumentList>>, Unset<RetType>>
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, Call<E>, EmptyArgumentList, ()> {
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
        Set<Call<E>>,
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
    CallBuilder<E, Set<Call<E>>, Set<ExecutionInput<Args>>, Set<ReturnType<R>>>
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

impl<E, Args, R> CallParams<E, Call<E>, Args, R>
where
    E: Environment,
{
    /// Returns the account ID of the called contract instance.
    #[inline]
    pub fn callee(&self) -> &E::AccountId {
        &self.call_type.callee
    }

    /// Returns the chosen ref time limit for the called contract execution.
    #[inline]
    pub fn ref_time_limit(&self) -> u64 {
        self.call_type.ref_time_limit
    }

    /// Returns the chosen proof size limit for the called contract execution.
    #[inline]
    pub fn proof_size_limit(&self) -> u64 {
        self.call_type.proof_size_limit
    }

    /// Returns the chosen storage deposit limit for the called contract execution.
    #[inline]
    pub fn storage_deposit_limit(&self) -> Option<&E::Balance> {
        self.call_type.storage_deposit_limit.as_ref()
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

impl<E, Args, R> CallParams<E, Call<E>, Args, R>
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
        crate::invoke_contract(self)
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
        crate::invoke_contract(self)
    }
}
