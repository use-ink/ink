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
mod delegate;

pub use call::Call;
pub use delegate::DelegateCall;

use core::marker::PhantomData;

use ink_primitives::{
    Address,
    abi::{
        Ink,
        Sol,
    },
};

use crate::{
    call::{
        Execution,
        ExecutionInput,
        utils::{
            EmptyArgumentList,
            ReturnType,
            Set,
            Unset,
        },
    },
    types::Environment,
};

/// The final parameters to the cross-contract call.
#[derive(Debug)]
pub struct CallParams<E, CallType, Args, R, Abi>
where
    E: Environment,
{
    /// A marker to indicate which type of call to perform.
    call_type: CallType,
    /// The expected return type.
    _return_type: ReturnType<R>,
    /// The inputs to the execution which is a selector and encoded arguments.
    exec_input: ExecutionInput<Args, Abi>,
    /// `Environment` is used by `CallType` for correct types
    _phantom: PhantomData<fn() -> E>,
}

impl<E, CallType, Args, R, Abi> CallParams<E, CallType, Args, R, Abi>
where
    E: Environment,
{
    /// Returns the execution input.
    #[inline]
    pub fn exec_input(&self) -> &ExecutionInput<Args, Abi> {
        &self.exec_input
    }
}

/// Returns a new [`CallBuilder`] to build up the parameters to a cross-contract call
/// that uses the "default" ABI for calls for the ink! project.
///
/// # Note
///
/// The "default" ABI for calls is "ink", unless the ABI is set to "sol"
/// in the ink! project's manifest file (i.e. `Cargo.toml`).
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
/// # use ink_primitives::Address;
///
/// type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// build_call::<DefaultEnvironment>()
///     .call(Address::from([0x42; 20]))
///     .ref_time_limit(5000)
///     .transferred_value(ink::U256::from(10))
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
///     .call_type(Call::new(ink::Address::from([0x42; 20])))
///     .ref_time_limit(5000)
///     .transferred_value(ink::U256::from(10))
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
/// use ink::Address;
/// # use ink_primitives::Clear;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let my_return_value: i32 = build_call::<DefaultEnvironment>()
///     .delegate(Address::zero())
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
/// # use ink_primitives::Address;
///
/// type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// let call_result = build_call::<DefaultEnvironment>()
///     .call(Address::from([0x42; 20]))
///     .ref_time_limit(5000)
///     .transferred_value(ink::U256::from(10))
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
    Unset<Call>,
    Unset<ExecutionInput<EmptyArgumentList<crate::DefaultAbi>, crate::DefaultAbi>>,
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

/// Returns a new [`CallBuilder`] to build up the parameters to a cross-contract call
/// that uses ink! ABI Encoding (i.e. with SCALE codec for input/output encode/decode).
///
/// See [`build_call`] for more details on usage.
#[allow(clippy::type_complexity)]
pub fn build_call_ink<E>() -> CallBuilder<
    E,
    Unset<Call>,
    Unset<ExecutionInput<EmptyArgumentList<Ink>, Ink>>,
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

/// Returns a new [`CallBuilder`] to build up the parameters to a cross-contract call
/// that uses Solidity ABI Encoding.
///
/// See [`build_call`] for more details on usage.
#[allow(clippy::type_complexity)]
pub fn build_call_sol<E>() -> CallBuilder<
    E,
    Unset<Call>,
    Unset<ExecutionInput<EmptyArgumentList<Sol>, Sol>>,
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
    _phantom: PhantomData<fn() -> E>, // todo possibly remove?
}

impl<E, Args, RetType, Abi> From<Execution<Args, RetType, Abi>>
    for CallBuilder<
        E,
        Unset<Call>,
        Set<ExecutionInput<Args, Abi>>,
        Set<ReturnType<RetType>>,
    >
where
    E: Environment,
{
    fn from(invoke: Execution<Args, RetType, Abi>) -> Self {
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

impl<E, CallType, RetType, Abi>
    CallBuilder<E, CallType, Unset<ExecutionInput<EmptyArgumentList<Abi>, Abi>>, RetType>
where
    E: Environment,
{
    /// Sets the execution input to the given value.
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args, Abi>,
    ) -> CallBuilder<E, CallType, Set<ExecutionInput<Args, Abi>>, RetType> {
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
    /// Prepares the `CallBuilder` for a cross-contract [`Call`] to the latest `call_v2`
    /// host function.
    pub fn call(self, callee: Address) -> CallBuilder<E, Set<Call>, Args, RetType> {
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
        address: Address,
    ) -> CallBuilder<E, Set<DelegateCall>, Args, RetType> {
        CallBuilder {
            // todo Generic `Set` can be removed
            call_type: Set(DelegateCall::new(address)),
            exec_input: self.exec_input,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}
