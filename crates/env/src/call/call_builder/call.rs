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
    U256,
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
    types::{
        Environment,
        Gas,
    },
};

/// The default call type for cross-contract calls, for calling into the latest `call`
/// host function. This adds the additional weight limit parameter `proof_size_limit` as
/// well as `storage_deposit_limit`.
#[derive(Clone)]
pub struct Call {
    callee: Address,
    ref_time_limit: u64,
    proof_size_limit: u64,
    storage_deposit_limit: Option<U256>,
    transferred_value: U256,
    call_flags: CallFlags,
}

impl Call {
    /// Returns a clean builder for [`Call`].
    pub fn new(callee: Address) -> Self {
        Self {
            callee,
            ref_time_limit: u64::MAX,
            proof_size_limit: u64::MAX,
            storage_deposit_limit: None,
            transferred_value: U256::zero(),
            call_flags: CallFlags::empty(),
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<Call>, Args, RetType>
where
    E: Environment,
{
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
    pub fn storage_deposit_limit(self, storage_deposit_limit: U256) -> Self {
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
    pub fn transferred_value(self, transferred_value: U256) -> Self {
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

impl<E, Args, RetType, Abi>
    CallBuilder<E, Set<Call>, Set<ExecutionInput<Args, Abi>>, Set<ReturnType<RetType>>>
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, Call, Args, RetType, Abi> {
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
        Set<Call>,
        Unset<ExecutionInput<EmptyArgumentList<Abi>, Abi>>,
        Unset<RetType>,
    >
where
    E: Environment,
    Abi: Default,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, Call, EmptyArgumentList<Abi>, (), Abi> {
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
        Set<Call>,
        Unset<ExecutionInput<EmptyArgumentList<Abi>, Abi>>,
        Unset<ReturnType<()>>,
    >
where
    E: Environment,
    EmptyArgumentList<Abi>: AbiEncodeWith<Abi>,
    (): DecodeMessageResult<Abi>,
    Abi: Default,
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

impl<E, Args, R, Abi>
    CallBuilder<E, Set<Call>, Set<ExecutionInput<Args, Abi>>, Set<ReturnType<R>>>
where
    E: Environment,
    Args: AbiEncodeWith<Abi>,
    R: DecodeMessageResult<Abi>,
    Abi: Default,
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

impl<E, Args, R, Abi> CallParams<E, Call, Args, R, Abi>
where
    E: Environment,
{
    /// Returns the contract address of the called contract instance.
    #[inline]
    pub fn callee(&self) -> &Address {
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
    /// todo
    #[inline]
    pub fn storage_deposit_limit(&self) -> Option<U256> {
        self.call_type.storage_deposit_limit
    }

    /// Returns the transferred value for the called contract.
    #[inline]
    pub fn transferred_value(&self) -> &U256 {
        &self.call_type.transferred_value
    }

    /// Returns the call flags.
    #[inline]
    pub fn call_flags(&self) -> &CallFlags {
        &self.call_type.call_flags
    }
}

impl<E, Args, R, Abi> CallParams<E, Call, Args, R, Abi>
where
    E: Environment,
    Args: AbiEncodeWith<Abi>,
    R: DecodeMessageResult<Abi>,
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
