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
    Clear,
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
    pub(crate) fn call_flags(&self) -> &CallFlags {
        &self.call_flags
    }

    /// Returns the execution input.
    #[inline]
    pub(crate) fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

impl<E, Args, R> CallParams<E, Call<E>, Args, R>
where
    E: Environment,
{
    /// Returns the account ID of the called contract instance.
    #[inline]
    pub(crate) fn callee(&self) -> &E::AccountId {
        &self
            .call_type
            .callee
            .as_ref()
            .expect("TODO, probably return Option here")
    }

    /// Returns the chosen gas limit for the called contract execution.
    #[inline]
    pub(crate) fn gas_limit(&self) -> Gas {
        self.call_type.gas_limit
    }

    /// Returns the transferred value for the called contract.
    #[inline]
    pub(crate) fn transferred_value(&self) -> &E::Balance {
        &self.call_type.transferred_value
    }
}

impl<E, Args, R> CallParams<E, DelegateCall<E>, Args, R>
where
    E: Environment,
{
    /// Returns the code hash which we use to perform a delegate call.
    #[inline]
    pub(crate) fn code_hash(&self) -> &E::Hash {
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
    pub fn invoke(&self) -> Result<R, crate::Error> {
        crate::invoke_contract(self)
    }
}

impl<E, Args, R> CallParams<E, DelegateCall<E>, Args, R>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Invokes the contract via delegated call with the given
    /// built-up call parameters.
    ///
    /// Returns the result of the contract execution.
    pub fn invoke(&self) -> Result<R, crate::Error> {
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
/// - receives the following arguments in order
///    1. an `i32` with value `42`
///    2. a `bool` with value `true`
///    3. an array of 32 `u8` with value `0x10`
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
///     .call_type(
///             Call::new()
///                 .callee(AccountId::from([0x42; 32]))
///                 .gas_limit(5000)
///                 .transferred_value(10))
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .returns::<()>()
///     .fire()
///     .unwrap();
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
/// - receives the following arguments in order
///    1. an `i32` with value `42`
///    2. a `bool` with value `true`
///    3. an array of 32 `u8` with value `0x10`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput, Call},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .call_type(Call::new()
///                 .callee(AccountId::from([0x42; 32]))
///                 .gas_limit(5000))
///     .transferred_value(10)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .returns::<i32>()
///     .fire()
///     .unwrap();
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
/// #     Clear,
/// #     call::{build_call, Selector, ExecutionInput, utils::ReturnType, DelegateCall},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .call_type(DelegateCall::new()
///                 .code_hash(<DefaultEnvironment as Environment>::Hash::clear()))
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .returns::<i32>()
///     .fire()
///     .unwrap();
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

/// The default call type for cross-contract calls. Performs a cross-contract call to `callee`
/// with gas limit `gas_limit`, transferring `transferred_value` of currency.
pub struct Call<E: Environment> {
    callee: Option<E::AccountId>,
    gas_limit: Gas,
    transferred_value: E::Balance,
}

impl<E: Environment> Default for Call<E> {
    fn default() -> Self {
        Call {
            callee: Default::default(),
            gas_limit: Default::default(),
            transferred_value: E::Balance::zero(),
        }
    }
}

impl<E: Environment> Call<E> {
    /// Returns a clean builder for [`Call`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl<E> Call<E>
where
    E: Environment,
{
    /// Sets the `callee` for the current cross-contract call.
    pub fn callee(self, callee: E::AccountId) -> Self {
        Call {
            callee: Some(callee),
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
        }
    }

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
    pub fn new() -> Self {
        Default::default()
    }
}

impl<E: Environment> Default for DelegateCall<E> {
    fn default() -> Self {
        DelegateCall {
            code_hash: E::Hash::clear(),
        }
    }
}

impl<E: Environment> DelegateCall<E> {
    /// Sets the `code_hash` to perform a delegate call with.
    pub fn code_hash(self, code_hash: E::Hash) -> Self {
        DelegateCall { code_hash }
    }
}

/// Builds up a cross contract call.
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

impl<E, Args, RetType> CallBuilder<E, Set<Call<E>>, Args, RetType>
where
    E: Environment,
{
    /// Sets the `callee` for the current cross-contract call.
    pub fn callee(self, callee: E::AccountId) -> Self {
        let call_type = self.call_type.value();
        CallBuilder {
            call_type: Set(Call {
                callee: Some(callee),
                gas_limit: call_type.gas_limit,
                transferred_value: call_type.transferred_value,
            }),
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }

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
    pub fn fire(self) -> Result<(), Error> {
        self.params().invoke()
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
    /// Invokes the cross-chain function call.
    pub fn fire(self) -> Result<(), Error> {
        self.params().invoke()
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
    pub fn fire(self) -> Result<R, Error> {
        self.params().invoke()
    }
}

impl<E, Args, R>
    CallBuilder<E, Set<DelegateCall<E>>, Set<ExecutionInput<Args>>, Set<ReturnType<R>>>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Invokes the cross-chain function call and returns the result.
    pub fn fire(self) -> Result<R, Error> {
        self.params().invoke()
    }
}
