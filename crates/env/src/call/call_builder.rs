// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
        utils::{
            EmptyArgumentList,
            ReturnType,
            Set,
            Unset,
            Unwrap,
        },
        ExecutionInput,
    },
    Environment,
    Error,
};
use core::marker::PhantomData;

/// The final parameters to the cross-contract call.
#[derive(Debug)]
pub struct CallParams<E, Args, R>
where
    E: Environment,
{
    /// The account ID of the to-be-called smart contract.
    callee: E::AccountId,
    /// The maximum gas costs allowed for the call.
    gas_limit: u64,
    /// The transferred value for the call.
    transferred_value: E::Balance,
    /// The expected return type.
    return_type: ReturnType<R>,
    /// The inputs to the execution which is a selector and encoded arguments.
    exec_input: ExecutionInput<Args>,
}

#[cfg(
    // We do not currently support cross-contract calling in the off-chain
    // environment so we do not have to provide these getters in case of
    // off-chain environment compilation.
    all(not(feature = "std"), target_arch = "wasm32")
)]
impl<E, Args, R> CallParams<E, Args, R>
where
    E: Environment,
{
    /// Returns the account ID of the called contract instance.
    #[inline]
    pub(crate) fn callee(&self) -> &E::AccountId {
        &self.callee
    }

    /// Returns the chosen gas limit for the called contract execution.
    #[inline]
    pub(crate) fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    /// Returns the transferred value for the called contract.
    #[inline]
    pub(crate) fn transferred_value(&self) -> &E::Balance {
        &self.transferred_value
    }

    /// Returns the execution input.
    #[inline]
    pub(crate) fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }
}

impl<E, Args> CallParams<E, Args, ()>
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

impl<E, Args, R> CallParams<E, Args, ReturnType<R>>
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
/// not return any value back to its caller. The called function ...
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
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// build_call::<DefaultEnvironment>()
///     .callee(AccountId::from([0x42; 32]))
///     .gas_limit(5000)
///     .transferred_value(10)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42)
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
/// return a `i32` value back to its caller. The called function ...
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
/// #     call::{build_call, Selector, ExecutionInput, utils::ReturnType},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .callee(AccountId::from([0x42; 32]))
///     .gas_limit(5000)
///     .transferred_value(10)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42)
///             .push_arg(true)
///             .push_arg(&[0x10; 32])
///     )
///     .returns::<ReturnType<i32>>()
///     .fire()
///     .unwrap();
/// ```
#[allow(clippy::type_complexity)]
pub fn build_call<E>() -> CallBuilder<
    E,
    Unset<E::AccountId>,
    Unset<u64>,
    Unset<E::Balance>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    Unset<ReturnType<()>>,
>
where
    E: Environment,
{
    CallBuilder {
        env: Default::default(),
        callee: Default::default(),
        gas_limit: Default::default(),
        transferred_value: Default::default(),
        exec_input: Default::default(),
        return_type: Default::default(),
    }
}

/// Builds up a cross contract call.
pub struct CallBuilder<E, Callee, GasLimit, TransferredValue, Args, RetType>
where
    E: Environment,
{
    env: PhantomData<fn() -> E>,
    /// The current parameters that have been built up so far.
    callee: Callee,
    gas_limit: GasLimit,
    transferred_value: TransferredValue,
    exec_input: Args,
    return_type: RetType,
}

impl<E, GasLimit, TransferredValue, Args, RetType>
    CallBuilder<E, Unset<E::AccountId>, GasLimit, TransferredValue, Args, RetType>
where
    E: Environment,
{
    /// Sets the called smart contract instance account ID to the given value.
    #[inline]
    pub fn callee(
        self,
        callee: E::AccountId,
    ) -> CallBuilder<E, Set<E::AccountId>, GasLimit, TransferredValue, Args, RetType>
    {
        CallBuilder {
            env: Default::default(),
            callee: Set(callee),
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
            exec_input: self.exec_input,
            return_type: self.return_type,
        }
    }
}

impl<E, Callee, TransferredValue, Args, RetType>
    CallBuilder<E, Callee, Unset<u64>, TransferredValue, Args, RetType>
where
    E: Environment,
{
    /// Sets the maximumly allowed gas costs for the call.
    #[inline]
    pub fn gas_limit(
        self,
        gas_limit: u64,
    ) -> CallBuilder<E, Callee, Set<u64>, TransferredValue, Args, RetType> {
        CallBuilder {
            env: Default::default(),
            callee: self.callee,
            gas_limit: Set(gas_limit),
            transferred_value: self.transferred_value,
            exec_input: self.exec_input,
            return_type: self.return_type,
        }
    }
}

impl<E, Callee, GasLimit, Args, RetType>
    CallBuilder<E, Callee, GasLimit, Unset<E::Balance>, Args, RetType>
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn transferred_value(
        self,
        transferred_value: E::Balance,
    ) -> CallBuilder<E, Callee, GasLimit, Set<E::Balance>, Args, RetType> {
        CallBuilder {
            env: Default::default(),
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value: Set(transferred_value),
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

impl<E, Callee, GasLimit, TransferredValue, Args>
    CallBuilder<E, Callee, GasLimit, TransferredValue, Args, Unset<ReturnType<()>>>
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
    pub fn returns<R>(
        self,
    ) -> CallBuilder<E, Callee, GasLimit, TransferredValue, Args, Set<R>>
    where
        R: IndicateReturnType,
    {
        CallBuilder {
            env: Default::default(),
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
            exec_input: self.exec_input,
            return_type: Set(Default::default()),
        }
    }
}

impl<E, Callee, GasLimit, TransferredValue, RetType>
    CallBuilder<
        E,
        Callee,
        GasLimit,
        TransferredValue,
        Unset<ExecutionInput<EmptyArgumentList>>,
        RetType,
    >
where
    E: Environment,
{
    /// Sets the execution input to the given value.
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args>,
    ) -> CallBuilder<
        E,
        Callee,
        GasLimit,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        RetType,
    > {
        CallBuilder {
            env: Default::default(),
            callee: self.callee,
            gas_limit: self.gas_limit,
            transferred_value: self.transferred_value,
            exec_input: Set(exec_input),
            return_type: self.return_type,
        }
    }
}

impl<E, GasLimit, TransferredValue, Args, RetType>
    CallBuilder<
        E,
        Set<E::AccountId>,
        GasLimit,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<RetType>,
    >
where
    E: Environment,
    GasLimit: Unwrap<Output = u64>,
    TransferredValue: Unwrap<Output = E::Balance>,
{
    /// Finalizes the call builder to call a function.
    pub fn params(self) -> CallParams<E, Args, RetType> {
        CallParams {
            callee: self.callee.value(),
            gas_limit: self.gas_limit.unwrap_or_else(|| 0),
            transferred_value: self
                .transferred_value
                .unwrap_or_else(|| E::Balance::from(0u32)),
            return_type: Default::default(),
            exec_input: self.exec_input.value(),
        }
    }
}

impl<E, GasLimit, TransferredValue, Args>
    CallBuilder<
        E,
        Set<E::AccountId>,
        GasLimit,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<()>,
    >
where
    E: Environment,
    GasLimit: Unwrap<Output = u64>,
    Args: scale::Encode,
    TransferredValue: Unwrap<Output = E::Balance>,
{
    /// Invokes the cross-chain function call.
    pub fn fire(self) -> Result<(), Error> {
        self.params().invoke()
    }
}

impl<E, GasLimit, TransferredValue, Args, R>
    CallBuilder<
        E,
        Set<E::AccountId>,
        GasLimit,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<R>>,
    >
where
    E: Environment,
    GasLimit: Unwrap<Output = u64>,
    Args: scale::Encode,
    R: scale::Decode,
    TransferredValue: Unwrap<Output = E::Balance>,
{
    /// Invokes the cross-chain function call and returns the result.
    pub fn fire(self) -> Result<R, Error> {
        self.params().eval()
    }
}
