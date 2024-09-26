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

mod call;
// mod call_v1;
mod delegate;

pub use call::Call;
// pub use call_v1::CallV1;
pub use delegate::DelegateCall;

use crate::{
    call::{
        utils::{
            EmptyArgumentList,
            ReturnType,
            Set,
            Unset,
        },
        Execution,
        ExecutionInput,
    },
    Environment,
};
use core::marker::PhantomData;

/// The final parameters to the cross-contract call.
#[derive(Debug)]
pub struct CallParams<E, CallType, Args, R>
where
    E: Environment,
{
    /// A marker to indicate which type of call to perform.
    call_type: CallType,
    /// The expected return type.
    _return_type: ReturnType<R>,
    /// The inputs to the execution which is a selector and encoded arguments.
    exec_input: ExecutionInput<Args>,
    /// `Environment` is used by `CallType` for correct types
    _phantom: PhantomData<fn() -> E>,
}

impl<E, CallType, Args, R> CallParams<E, CallType, Args, R>
where
    E: Environment,
{
    /// Returns the execution input.
    #[inline]
    pub fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

/// Returns a new [`CallBuilder`] to build up the parameters to a cross-contract call.
///
/// # Example
///
/// **Note:** The shown examples panic because there is currently no cross-calling
///           support in the off-chain testing environment. However, this code
///           should work fine in on-chain environments.
///
/// ## Example 1: No Return Value
///
/// The below example shows calling of a message of another contract that does
/// not return any value back to its caller. The called function:
///
/// - has a selector equal to `0xDEADBEEF`
/// - is provided with 5000 units of gas for its execution
/// - is provided with 10 units of transferred value for the contract instance
/// - receives the following arguments in order 1. an `i32` with value `42` 2. a `bool`
///   with value `true` 3. an array of 32 `u8` with value `0x10`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput}
/// # };
/// # use ink_env::call::CallV1;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// build_call::<DefaultEnvironment>()
///     .call_v1(AccountId::from([0x42; 32]))
///     .gas_limit(5000)
///     .transferred_value(10)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32]),
///     )
///     .returns::<()>()
///     .invoke();
/// ```
///
/// ## Example 2: With Return Value
///
/// The below example shows calling of a message of another contract that does
/// return a `i32` value back to its caller. The called function:
///
/// - has a selector equal to `0xDEADBEEF`
/// - is provided with 5000 units of gas for its execution
/// - is provided with 10 units of transferred value for the contract instance
/// - receives the following arguments in order 1. an `i32` with value `42` 2. a `bool`
///   with value `true` 3. an array of 32 `u8` with value `0x10`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput, CallV1},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .call_type(CallV1::new(AccountId::from([0x42; 32])))
///     .gas_limit(5000)
///     .transferred_value(10)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32]),
///     )
///     .returns::<i32>()
///     .invoke();
/// ```
///
/// ## Example 3: Delegate call
///
/// **Note:** The shown example panics because there is currently no delegate calling
///           support in the off-chain testing environment. However, this code
///           should work fine in on-chain environments.
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput, utils::ReturnType, DelegateCall},
/// # };
/// # use ink_primitives::Clear;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .delegate(<DefaultEnvironment as Environment>::Hash::CLEAR_HASH)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .returns::<i32>()
///     .invoke();
/// ```
///
/// # Handling `LangError`s
///
/// It is also important to note that there are certain types of errors which can happen
/// during cross-contract calls which can be handled know as
/// [`LangError`][`ink_primitives::LangError`].
///
/// If you want to handle these errors use the [`CallBuilder::try_invoke`] methods instead
/// of the [`CallBuilder::invoke`] ones.
///
/// **Note:** The shown examples panic because there is currently no cross-calling
///           support in the off-chain testing environment. However, this code
///           should work fine in on-chain environments.
///
/// ## Example: Handling a `LangError`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput}
/// # };
/// # use ink_env::call::CallV1;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// let call_result = build_call::<DefaultEnvironment>()
///     .call_v1(AccountId::from([0x42; 32]))
///     .gas_limit(5000)
///     .transferred_value(10)
///     .try_invoke()
///     .expect("Got an error from the Contract's pallet.");
///
/// match call_result {
///     Ok(_) => unimplemented!(),
///     Err(e @ ink_primitives::LangError::CouldNotReadInput) => unimplemented!(),
///     Err(_) => unimplemented!(),
/// }
/// ```
#[allow(clippy::type_complexity)]
pub fn build_call<E>() -> CallBuilder<
    E,
    Unset<Call<E>>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    Unset<ReturnType<()>>,
>
where
    E: Environment,
{
    CallBuilder {
        call_type: Default::default(),
        exec_input: Default::default(),
        return_type: Default::default(),
        _phantom: Default::default(),
    }
}

/// Builds up a cross contract call.
#[derive(Clone)]
pub struct CallBuilder<E, CallType, Args, RetType>
where
    E: Environment,
{
    /// The current parameters that have been built up so far.
    call_type: CallType,
    exec_input: Args,
    return_type: RetType,
    _phantom: PhantomData<fn() -> E>,
}

impl<E, Args, RetType> From<Execution<Args, RetType>>
    for CallBuilder<
        E,
        Unset<Call<E>>,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<RetType>>,
    >
where
    E: Environment,
{
    fn from(invoke: Execution<Args, RetType>) -> Self {
        CallBuilder {
            call_type: Default::default(),
            exec_input: Set(invoke.input),
            return_type: Set(invoke.output),
            _phantom: Default::default(),
        }
    }
}

impl<E, CallType, Args, RetType> CallBuilder<E, Unset<CallType>, Args, RetType>
where
    E: Environment,
{
    /// The type of the call.
    #[inline]
    #[must_use]
    pub fn call_type<NewCallType>(
        self,
        call_type: NewCallType,
    ) -> CallBuilder<E, Set<NewCallType>, Args, RetType> {
        CallBuilder {
            call_type: Set(call_type),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, CallType, Args> CallBuilder<E, CallType, Args, Unset<ReturnType<()>>>
where
    E: Environment,
{
    /// Sets the type of the returned value upon the execution of the call.
    ///
    /// # Note
    ///
    /// Either use `.returns::<()>` to signal that the call does not return a value
    /// or use `.returns::<T>` to signal that the call returns a value of type `T`.
    #[inline]
    pub fn returns<R>(self) -> CallBuilder<E, CallType, Args, Set<ReturnType<R>>> {
        CallBuilder {
            call_type: self.call_type,
            exec_input: self.exec_input,
            return_type: Set(Default::default()),
            _phantom: Default::default(),
        }
    }
}

impl<E, CallType, RetType>
    CallBuilder<E, CallType, Unset<ExecutionInput<EmptyArgumentList>>, RetType>
where
    E: Environment,
{
    /// Sets the execution input to the given value.
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args>,
    ) -> CallBuilder<E, CallType, Set<ExecutionInput<Args>>, RetType> {
        CallBuilder {
            call_type: self.call_type,
            exec_input: Set(exec_input),
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, CallType, Args, RetType> CallBuilder<E, Unset<CallType>, Args, RetType>
where
    E: Environment,
{
    /// Prepares the `CallBuilder` for a cross-contract [`CallV1`], calling into the
    /// original `call` host function.
    // pub fn call_v1(
    //     self,
    //     callee: E::AccountId,
    // ) -> CallBuilder<E, Set<CallV1<E>>, Args, RetType> {
    //     CallBuilder {
    //         call_type: Set(CallV1::new(callee)),
    //         exec_input: self.exec_input,
    //         return_type: self.return_type,
    //         _phantom: Default::default(),
    //     }
    // }

    /// Prepares the `CallBuilder` for a cross-contract [`Call`] to the latest `call_v2`
    /// host function.
    pub fn call(
        self,
        callee: E::AccountId,
    ) -> CallBuilder<E, Set<Call<E>>, Args, RetType> {
        CallBuilder {
            call_type: Set(Call::new(callee)),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }

    /// Prepares the `CallBuilder` for a cross-contract [`DelegateCall`].
    pub fn delegate(
        self,
        code_hash: E::Hash,
    ) -> CallBuilder<E, Set<DelegateCall<E>>, Args, RetType> {
        CallBuilder {
            call_type: Set(DelegateCall::new(code_hash)),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}
