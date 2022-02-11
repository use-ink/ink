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
    env: PhantomData<fn() -> E>,
    /// A marker to tell us which call we are going to perform.
    call_type: CallType,
    /// The flags used to change the behavior of a contract call.
    call_flags: CallFlags,
    /// The expected return type.
    _return_type: ReturnType<R>,
    /// The inputs to the execution which is a selector and encoded arguments.
    exec_input: ExecutionInput<Args>,
}

impl<E, Args, R> CallParams<E, Call<E, E::AccountId, u64, E::Balance>, Args, R>
where
    E: Environment,
{
    /// Returns the account ID of the called contract instance.
    #[inline]
    pub(crate) fn callee(&self) -> &E::AccountId {
        &self.call_type.callee
    }

    /// Returns the call flags.
    #[inline]
    pub(crate) fn call_flags(&self) -> &CallFlags {
        &self.call_flags
    }

    /// Returns the chosen gas limit for the called contract execution.
    #[inline]
    pub(crate) fn gas_limit(&self) -> u64 {
        self.call_type.gas_limit
    }

    /// Returns the transferred value for the called contract.
    #[inline]
    pub(crate) fn transferred_value(&self) -> &E::Balance {
        &self.call_type.transferred_value
    }

    /// Returns the execution input.
    #[inline]
    pub(crate) fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

impl<E, Args, R> CallParams<E, DelegateCall<E, E::Hash>, Args, R>
where
    E: Environment,
{
    /// Returns the call flags.
    #[inline]
    #[allow(dead_code)] // it is used in the `invoke_contract_delegate` / `eval_contract_delegate` functions.
    pub(crate) fn call_flags(&self) -> &CallFlags {
        &self.call_flags
    }

    /// Returns the code hash which we use to perform a delegate call.
    #[inline]
    #[allow(dead_code)] // it is used in the `invoke_contract_delegate` / `eval_contract_delegate` functions.
    pub(crate) fn code_hash(&self) -> &E::Hash {
        &self.call_type.code_hash
    }

    /// Returns the execution input.
    #[inline]
    #[allow(dead_code)] // it is used in the `invoke_contract_delegate` / `eval_contract_delegate` functions.
    pub(crate) fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

impl<E, Args> CallParams<E, Call<E, E::AccountId, u64, E::Balance>, Args, ()>
where
    E: Environment,
    Args: scale::Encode,
{
    /// Invokes the contract with the given built-up call parameters.
    ///
    /// # Note
    ///
    /// Prefer [`invoke`](`Self::invoke`) over [`eval`](`Self::eval`) if the
    /// called contract message does not return anything because it is more efficient.
    pub fn invoke(&self) -> Result<(), crate::Error> {
        crate::invoke_contract(self)
    }
}

impl<E, Args> CallParams<E, DelegateCall<E, E::Hash>, Args, ()>
where
    E: Environment,
    Args: scale::Encode,
{
    /// Invokes the contract with the given built-up call parameters.
    ///
    /// # Note
    ///
    /// Prefer [`invoke`](`Self::invoke`) over [`eval`](`Self::eval`) if the
    /// called contract message does not return anything because it is more efficient.
    pub fn invoke(&self) -> Result<(), crate::Error> {
        crate::invoke_contract_delegate(self)
    }
}

impl<E, Args, R>
    CallParams<E, Call<E, E::AccountId, u64, E::Balance>, Args, ReturnType<R>>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Evaluates the contract with the given built-up call parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Note
    ///
    /// Prefer [`invoke`](`Self::invoke`) over [`eval`](`Self::eval`) if the
    /// called contract message does not return anything because it is more efficient.
    pub fn eval(&self) -> Result<R, crate::Error> {
        crate::eval_contract(self)
    }
}

impl<E, Args, R> CallParams<E, DelegateCall<E, E::Hash>, Args, ReturnType<R>>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Evaluates the contract with the given built-up call parameters.
    ///
    /// Returns the result of the contract execution.
    ///
    /// # Note
    ///
    /// Prefer [`invoke`](`Self::invoke`) over [`eval`](`Self::eval`) if the
    /// called contract message does not return anything because it is more efficient.
    pub fn eval(&self) -> Result<R, crate::Error> {
        crate::eval_contract_delegate(self)
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
///     .set_call_type(Call::new().set_callee(AccountId::from([0x42; 32])).set_gas_limit(5000).set_transferred_value(10))
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
/// #     call::{build_call, Selector, ExecutionInput, utils::ReturnType, Call},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .set_call_type(Call::new().set_callee(AccountId::from([0x42; 32]))
///                 .set_gas_limit(5000)
///                 .set_transferred_value(10))
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .returns::<ReturnType<i32>>()
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
///     .set_call_type(DelegateCall::new().set_code_hash(<DefaultEnvironment as Environment>::Hash::clear()))
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42u8)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .returns::<ReturnType<i32>>()
///     .fire()
///     .unwrap();
/// ```
#[allow(clippy::type_complexity)]
pub fn build_call<E>() -> CallBuilder<
    E,
    Unset<Call<E, E::AccountId, u64, E::Balance>>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    Unset<ReturnType<()>>,
>
where
    E: Environment,
{
    CallBuilder {
        env: Default::default(),
        call_type: Default::default(),
        call_flags: Default::default(),
        exec_input: Default::default(),
        return_type: Default::default(),
    }
}

/// The default call type for cross-contract calls. Performs a cross-contract call to `callee`
/// with gas limit `gas_limit`, transferring `transferred_value` of currency.
pub struct Call<E, Callee, GasLimit, TransferredValue> {
    env: PhantomData<fn() -> E>,
    callee: Callee,
    gas_limit: GasLimit,
    transferred_value: TransferredValue,
}

impl<E: Environment> Default for Call<E, E::AccountId, u64, E::Balance> {
    fn default() -> Self {
        Call {
            env: Default::default(),
            callee: Default::default(),
            gas_limit: Default::default(),
            transferred_value: E::Balance::zero(),
        }
    }
}

impl<E, GasLimit, TransferredValue> Call<E, E::AccountId, GasLimit, TransferredValue>
where
    E: Environment,
{
    /// Sets the `callee` for the current cross-contract call.
    pub fn set_callee(
        self,
        callee: E::AccountId,
    ) -> Call<E, E::AccountId, GasLimit, TransferredValue> {
        Call {
            env: self.env,
            callee,
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
        }
    }
}

impl<E, Callee, TransferredValue> Call<E, Callee, u64, TransferredValue>
where
    E: Environment,
{
    /// Sets the `gas_limit` for the current cross-contract call.
    pub fn set_gas_limit(self, gas_limit: u64) -> Call<E, Callee, u64, TransferredValue> {
        Call {
            env: self.env,
            callee: self.callee,
            gas_limit,
            transferred_value: self.transferred_value,
        }
    }
}

impl<E, Callee, GasLimit> Call<E, Callee, GasLimit, E::Balance>
where
    E: Environment,
{
    /// Sets the `transferred_value` for the current cross-contract call.
    pub fn set_transferred_value(
        self,
        transferred_value: E::Balance,
    ) -> Call<E, Callee, GasLimit, E::Balance> {
        Call {
            env: self.env,
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value,
        }
    }
}

impl<E: Environment> Call<E, E::AccountId, u64, E::Balance> {
    /// Returns a clean builder for [`Call`].
    pub fn new() -> Self {
        Default::default()
    }
}

/// The `delegatecall` call type. Performs a call with the given code hash.
pub struct DelegateCall<E: Environment, CodeHash> {
    env: PhantomData<fn() -> E>,
    code_hash: CodeHash,
}

impl<E: Environment> DelegateCall<E, E::Hash> {
    /// Returns a clean builder for [`DelegateCall`]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<E: Environment> Default for DelegateCall<E, E::Hash> {
    fn default() -> Self {
        DelegateCall {
            env: PhantomData,
            code_hash: E::Hash::clear(),
        }
    }
}

impl<E: Environment> DelegateCall<E, E::Hash> {
    /// Sets the `code_hash` to perform a delegate call with.
    pub fn set_code_hash(self, code_hash: E::Hash) -> DelegateCall<E, E::Hash> {
        DelegateCall {
            env: PhantomData,
            code_hash,
        }
    }
}

/// Builds up a cross contract call.
pub struct CallBuilder<E, CallType, Args, RetType>
where
    E: Environment,
{
    env: PhantomData<fn() -> E>,
    /// The current parameters that have been built up so far.
    call_type: CallType,
    call_flags: CallFlags,
    exec_input: Args,
    return_type: RetType,
}

impl<E, CallType, Args, RetType> CallBuilder<E, Unset<CallType>, Args, RetType>
where
    E: Environment,
{
    /// The type of the call.
    #[inline]
    #[must_use]
    pub fn set_call_type<NewCallType>(
        self,
        call_type: NewCallType,
    ) -> CallBuilder<E, Set<NewCallType>, Args, RetType> {
        CallBuilder {
            env: Default::default(),
            call_type: Set(call_type),
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
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
            env: Default::default(),
            call_type: self.call_type,
            call_flags,
            exec_input: self.exec_input,
            return_type: self.return_type,
        }
    }
}

mod seal {
    /// Used to prevent users from implementing `IndicateReturnType` for their own types.
    pub trait Sealed {}
    impl Sealed for () {}
    impl<T> Sealed for super::ReturnType<T> {}
}

/// Types that can be used in [`CallBuilder::returns`] to signal return type.
pub trait IndicateReturnType: Default + self::seal::Sealed {}
impl IndicateReturnType for () {}
impl<T> IndicateReturnType for ReturnType<T> {}

impl<E, CallType, Args> CallBuilder<E, CallType, Args, Unset<ReturnType<()>>>
where
    E: Environment,
{
    /// Sets the type of the returned value upon the execution of the call.
    ///
    /// # Note
    ///
    /// Either use `.returns::<()>` to signal that the call does not return a value
    /// or use `.returns::<ReturnType<T>>` to signal that the call returns a value of
    /// type `T`.
    #[inline]
    pub fn returns<R>(self) -> CallBuilder<E, CallType, Args, Set<R>>
    where
        R: IndicateReturnType,
    {
        CallBuilder {
            env: Default::default(),
            call_type: self.call_type,
            call_flags: self.call_flags,
            exec_input: self.exec_input,
            return_type: Set(Default::default()),
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
            env: Default::default(),
            call_type: self.call_type,
            call_flags: self.call_flags,
            exec_input: Set(exec_input),
            return_type: self.return_type,
        }
    }
}

impl<E, Args, RetType>
    CallBuilder<
        E,
        Set<Call<E, E::AccountId, u64, E::Balance>>,
        Set<ExecutionInput<Args>>,
        Set<RetType>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    #[allow(clippy::type_complexity)]
    pub fn params(
        self,
    ) -> CallParams<E, Call<E, E::AccountId, u64, E::Balance>, Args, RetType> {
        CallParams {
            env: self.env,
            call_type: self.call_type.value(),
            call_flags: self.call_flags,
            _return_type: Default::default(),
            exec_input: self.exec_input.value(),
        }
    }
}

impl<E, Args, RetType>
    CallBuilder<E, Set<DelegateCall<E, E::Hash>>, Set<ExecutionInput<Args>>, Set<RetType>>
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, DelegateCall<E, E::Hash>, Args, RetType> {
        CallParams {
            env: self.env,
            call_type: self.call_type.value(),
            call_flags: self.call_flags,
            _return_type: Default::default(),
            exec_input: self.exec_input.value(),
        }
    }
}

impl<E, RetType>
    CallBuilder<
        E,
        Set<Call<E, E::AccountId, u64, E::Balance>>,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Unset<RetType>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    #[allow(clippy::type_complexity)]
    pub fn params(
        self,
    ) -> CallParams<E, Call<E, E::AccountId, u64, E::Balance>, EmptyArgumentList, ()>
    {
        CallParams {
            env: self.env,
            call_type: self.call_type.value(),
            call_flags: self.call_flags,
            _return_type: Default::default(),
            exec_input: Default::default(),
        }
    }
}

impl<E, RetType>
    CallBuilder<
        E,
        Set<DelegateCall<E, E::Hash>>,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Unset<RetType>,
    >
where
    E: Environment,
{
    /// Finalizes the call builder to call a function.
    pub fn params(
        self,
    ) -> CallParams<E, DelegateCall<E, E::Hash>, EmptyArgumentList, ()> {
        CallParams {
            env: self.env,
            call_type: self.call_type.value(),
            call_flags: self.call_flags,
            _return_type: Default::default(),
            exec_input: Default::default(),
        }
    }
}

impl<E, Args>
    CallBuilder<
        E,
        Set<Call<E, E::AccountId, u64, E::Balance>>,
        Set<ExecutionInput<Args>>,
        Set<()>,
    >
where
    E: Environment,
    Args: scale::Encode,
{
    /// Invokes the cross-chain function call.
    pub fn fire(self) -> Result<(), Error> {
        self.params().invoke()
    }
}

impl<E, Args>
    CallBuilder<E, Set<DelegateCall<E, E::Hash>>, Set<ExecutionInput<Args>>, Set<()>>
where
    E: Environment,
    Args: scale::Encode,
{
    /// Invokes the cross-chain function call.
    pub fn fire(self) -> Result<(), Error> {
        self.params().invoke()
    }
}

impl<E>
    CallBuilder<
        E,
        Set<Call<E, E::AccountId, u64, E::Balance>>,
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
        Set<DelegateCall<E, E::Hash>>,
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
    CallBuilder<
        E,
        Set<Call<E, E::AccountId, u64, E::Balance>>,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<R>>,
    >
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Invokes the cross-chain function call and returns the result.
    pub fn fire(self) -> Result<R, Error> {
        self.params().eval()
    }
}

impl<E, Args, R>
    CallBuilder<
        E,
        Set<DelegateCall<E, E::Hash>>,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<R>>,
    >
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    /// Invokes the cross-chain function call and returns the result.
    pub fn fire(self) -> Result<R, Error> {
        self.params().eval()
    }
}
