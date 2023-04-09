// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
    backend::CallFlags,
    call::{
        utils::{
            EmptyArgumentList,
            ReturnType,
            Set,
            Unset,
        },
        ExecutionInput,
    },
    types::Gas,
    ContractEnv,
    Environment,
    Error,
};
use core::marker::PhantomData;
use num_traits::Zero;

/// The final parameters to the cross-contract call.
#[derive(Debug)]
pub struct CallParams<E, CallType, Args, R>
where
    E: Environment,
{
    /// A marker to indicate which type of call to perform.
    call_type: CallType,
    /// The flags used to change the behavior of a contract call.
    call_flags: CallFlags,
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
    /// Returns the call flags.
    #[inline]
    pub fn call_flags(&self) -> &CallFlags {
        &self.call_flags
    }

    /// Returns the execution input.
    #[inline]
    pub fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
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
/// # use ink_env::call::Call;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// build_call::<DefaultEnvironment>()
///     .call(AccountId::from([0x42; 32]))
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
/// #     call::{build_call, Selector, ExecutionInput, Call},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .call_type(Call::new(AccountId::from([0x42; 32])))
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
/// # use ink_env::call::Call;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// let call_result = build_call::<DefaultEnvironment>()
///     .call(AccountId::from([0x42; 32]))
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
        call_flags: Default::default(),
        exec_input: Default::default(),
        return_type: Default::default(),
        _phantom: Default::default(),
    }
}

/// The default call type for cross-contract calls. Performs a cross-contract call to
/// `callee` with gas limit `gas_limit`, transferring `transferred_value` of currency.
#[derive(Clone)]
pub struct Call<E: Environment> {
    callee: E::AccountId,
    gas_limit: Gas,
    transferred_value: E::Balance,
}

impl<E: Environment> Call<E> {
    /// Returns a clean builder for [`Call`].
    pub fn new(callee: E::AccountId) -> Self {
        Self {
            callee,
            gas_limit: Default::default(),
            transferred_value: E::Balance::zero(),
        }
    }
}

impl<E> Call<E>
where
    E: Environment,
{
    /// Sets the `gas_limit` for the current cross-contract call.
    pub fn gas_limit(self, gas_limit: Gas) -> Self {
        Call {
            callee: self.callee,
            gas_limit,
            transferred_value: self.transferred_value,
        }
    }

    /// Sets the `transferred_value` for the current cross-contract call.
    pub fn transferred_value(self, transferred_value: E::Balance) -> Self {
        Call {
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value,
        }
    }
}

/// The `delegatecall` call type. Performs a call with the given code hash.
pub struct DelegateCall<E: Environment> {
    code_hash: E::Hash,
}

impl<E: Environment> DelegateCall<E> {
    /// Returns a clean builder for [`DelegateCall`]
    pub const fn new(code_hash: E::Hash) -> Self {
        DelegateCall { code_hash }
    }
}

impl<E: Environment> DelegateCall<E> {
    /// Sets the `code_hash` to perform a delegate call with.
    pub fn code_hash(self, code_hash: E::Hash) -> Self {
        DelegateCall { code_hash }
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
    call_flags: CallFlags,
    exec_input: Args,
    return_type: RetType,
    _phantom: PhantomData<fn() -> E>,
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
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, CallType, Args, RetType> CallBuilder<E, CallType, Args, RetType>
where
    E: Environment,
{
    /// The flags used to change the behavior of the contract call.
    #[inline]
    #[must_use]
    pub fn call_flags(
        self,
        call_flags: CallFlags,
    ) -> CallBuilder<E, CallType, Args, RetType> {
        CallBuilder {
            call_type: self.call_type,
            call_flags,
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
            call_flags: self.call_flags,
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
            call_flags: self.call_flags,
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
    /// Prepares the `CallBuilder` for a cross-contract [`Call`].
    pub fn call(
        self,
        callee: E::AccountId,
    ) -> CallBuilder<E, Set<Call<E>>, Args, RetType> {
        CallBuilder {
            call_type: Set(Call::new(callee)),
            call_flags: self.call_flags,
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
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<Call<E>>, Args, RetType>
where
    E: Environment,
{
    /// Sets the `gas_limit` for the current cross-contract call.
    pub fn gas_limit(self, gas_limit: Gas) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                callee: call_type.callee,
                gas_limit,
                transferred_value: call_type.transferred_value,
            }),
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }

    /// Sets the `transferred_value` for the current cross-contract call.
    pub fn transferred_value(self, transferred_value: E::Balance) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                callee: call_type.callee,
                gas_limit: call_type.gas_limit,
                transferred_value,
            }),
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, Args, RetType> CallBuilder<E, Set<DelegateCall<E>>, Args, RetType>
where
    E: Environment,
{
    /// Sets the `code_hash` to perform a delegate call with.
    pub fn code_hash(self, code_hash: E::Hash) -> Self {
        CallBuilder {
            call_type: Set(DelegateCall { code_hash }),
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
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
            call_flags: self.call_flags,
            _return_type: Default::default(),
            exec_input: self.exec_input.value(),
            _phantom: self._phantom,
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
            call_flags: self.call_flags,
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
            call_flags: self.call_flags,
            _return_type: Default::default(),
            exec_input: Default::default(),
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
            call_flags: self.call_flags,
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

/// The analog of the [`build_call`] but from variable to get the [`ContractEnv::Env`].
#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub fn build_call_from_variable<V>(
    _: V,
) -> CallBuilder<
    V::Env,
    Unset<Call<V::Env>>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    Unset<ReturnType<()>>,
>
where
    V: ContractEnv,
{
    build_call::<V::Env>()
}

/// The helper macro reverses the order of the token stream.
///
/// ```rust
/// use ink_env::reverse_tokens_order;
/// assert_eq!(
///     core::any::TypeId::of::<u64>(),
///     reverse_tokens_order!(()>u64<::of::TypeId::any::core)
/// )
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! reverse_tokens_order {
    (@main_loop [ $( $heads:tt, )+ ],) => {
        $( $heads )*
    };
    (@main_loop [ $( $heads:tt, )+ ], $head:tt $( $tail:tt )*) => {
        $crate::reverse_tokens_order!(@main_loop [ $head, $( $heads, )+ ], $( $tail )*)
    };
    ($head:tt $( $tail:tt )*) => {
        $crate::reverse_tokens_order!(@main_loop [ $head, ], $( $tail )*)
    };
}

/// The primary internal implementation of the [`call_builder`](crate::call_builder)
/// macro. It parses the token stream and generates [`CallBuilder`].
///
/// The macro works as a state machine with the next steps:
/// - Reverses the token stream.
/// - Tries to find the template `(args)method.` or `(args, receiver)method::`.
/// - Collects arguments, receiver, and message descriptor getter.
/// - Reverse the token stream back.
/// - Expands into [`CallBuilder`].
#[doc(hidden)]
#[macro_export]
macro_rules! call_builder_inner {
    // Reverses the initial token stream to simplify handling in the `@reversed`.
    (@reverse $head:tt $( $tail:tt )*) => {
        $crate::call_builder_inner!(@reverse_main_loop [ $head, ], $( $tail )*)
    };
    (@reverse_main_loop [ $( $heads:tt, )+ ],) => {
        $crate::call_builder_inner!(@reversed $( $heads )*)
    };
    (@reverse_main_loop [ $( $heads:tt, )+ ], $head:tt $( $tail:tt )*) => {
        $crate::call_builder_inner!(@reverse_main_loop [ $head, $( $heads, )+ ], $( $tail )*)
    };

    // Entry point for the `... .method(inputs)`.
    // The `$input_bindings` are not reversed because it is part
    // of the `( $( $input_bindings:expr ),* )` that is single `tt`.
    (@reversed ( $( $input_bindings:expr ),* ) $method:ident . $( $rest:tt )+ ) => {
        $crate::call_builder_inner!(
            @final
            $crate::reverse_tokens_order!( $( $rest )+ ),
            $crate::reverse_tokens_order!( ( $( $input_bindings ),* ) $method . $( $rest )+ ),
            ::ink::codegen::paste! {
                $crate::reverse_tokens_order!( () [<__ink_ $method _description>] . $( $rest )+ )
            },
            $( $input_bindings )*
        )
    };

    // Entry point for the `... ::method(self, inputs)`.
    // The `$receiver` and `$input_bindings` are not reversed because it is part
    // of the `( $receiver:expr $(, $input_bindings:expr )* )` that is single `tt`.
    (@reversed ( $receiver:expr $(, $input_bindings:expr )* ) $method:ident :: $( $rest:tt )+ ) => {
        $crate::call_builder_inner!(
            @final
            $receiver,
            $crate::reverse_tokens_order!( ( $receiver $(, $input_bindings )* ) $method :: $( $rest )+ ),
            ::ink::codegen::paste! {
                $crate::reverse_tokens_order!( ($receiver) [<__ink_ $method _description>] :: $( $rest )+ )
            },
            $( $input_bindings )*
        )
    };

    // The final generated code by the macro.
    (@final $caller:expr, $call:expr, $description:expr, $( $input_bindings:expr)* ) => {{
        // Gets the message description with selector information.
        #[allow(clippy::unnecessary_mut_passed)]
        let message_description = $description;
        let call_builder = match $caller {
            ref caller => {
                // Creates the call builder with the selector from
                // the message descriptor with input arguments.
                let call_builder = $crate::call::build_call_from_variable(caller)
                    .call(::ink::ToAccountId::to_account_id(caller))
                    .exec_input(
                        $crate::call::ExecutionInput::new(
                            message_description.selector()
                        )
                        $(
                            .push_arg($input_bindings)
                        )*
                    )
                    .returns::<_>();

                call_builder
            }
        };

        // Forces setting of the return type of the call builder.
        if false {
            let _ = if false {
                $call
            } else {
                call_builder.invoke()
            };
            unreachable!();
        };
        call_builder
    }};
}

/// Returns a [`CallBuilder`] based on the message call signature.
///
///
/// ```should_panic
/// use ink::contract_ref;
/// use ink_env::{
///     call_builder,
///     CallFlags,
///     DefaultEnvironment,
/// };
/// use ink_primitives::AccountId;
///
/// #[ink::trait_definition]
/// pub trait Erc20 {
///     /// Returns the total supply of the ERC-20 smart contract.
///     #[ink(message)]
///     fn total_supply(&self) -> u128;
///
///     /// Transfers balance from the caller to the given address.
///     #[ink(message)]
///     fn transfer(&mut self, amount: u128, to: AccountId) -> bool;
/// }
///
/// let mut callee: contract_ref!(Erc20, DefaultEnvironment) =
///     AccountId::from([0; 32]).into();
/// call_builder!(callee.total_supply())
///     .transferred_value(1000)
///     .invoke();
/// call_builder!(callee.transfer(20, AccountId::from([1; 32])))
///     .call_flags(CallFlags::default().set_allow_reentry(true))
///     .invoke();
/// let ink_err = call_builder!(callee.total_supply()).try_invoke();
/// let message_err = ink_err.unwrap();
/// let supply = message_err.unwrap();
///
/// // Other supported syntax
/// call_builder!(Erc20::total_supply(&callee)).invoke();
/// call_builder!(Erc20::transfer(&mut callee, 20, AccountId::from([2; 32]))).invoke();
/// call_builder!(<_ as Erc20>::total_supply(&callee)).invoke();
/// call_builder!(<_ as Erc20>::transfer(
///     &mut callee,
///     20,
///     AccountId::from([3; 32])
/// ))
/// .invoke();
/// call_builder!(Erc20::total_supply(&callee)).invoke();
/// ```
#[macro_export]
macro_rules! call_builder {
( $( $tokens:tt )* ) => {{
        // Forces the compiler to check first that the expression is valid.
        let _ = || { $( $tokens )* };
        $crate::call_builder_inner!(@reverse $( $tokens )* )
    }};
}
