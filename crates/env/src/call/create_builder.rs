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
        utils::{
            EmptyArgumentList,
            ReturnType,
            Set,
            Unset,
        },
        ExecutionInput,
        Selector,
    },
    ContractEnv,
    Environment,
    Error,
};
use core::marker::PhantomData;

pub mod state {
    //! Type states that tell what state of a instantiation argument has not
    //! yet been set properly for a valid construction.

    /// Type state for the salt used for contract instantiation.
    pub enum Salt {}
}

/// Contracts that can be constructed from an `AccountId`.
///
/// # Note
///
/// This is needed because of conflicting implementations of `From<T> for T`
/// in the generated code of `ink`.
///
/// But it is possible to use `From<AccountId> for T` with [`crate::AccountIdGuard`]
/// bound.
pub trait FromAccountId<T>
where
    T: Environment,
{
    /// Creates the contract instance from the account ID of the already instantiated
    /// contract.
    fn from_account_id(account_id: <T as Environment>::AccountId) -> Self;
}

/// Represents any type that can be returned from an `ink!` constructor. The following
/// contract implements the four different return type signatures implementing this trait:
///
/// - `Self`
/// - `Result<Self, Error>`
/// - `Contract`
/// - `Result<Contract, Error>`
///
/// ```rust
/// #[ink::contract]
/// mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     #[derive(Debug, PartialEq, Eq)]
///     #[ink::scale_derive(Encode, Decode, TypeInfo)]
///     pub enum Error {
///         Foo,
///     }
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn new_self() -> Self {
///             Self {}
///         }
///
///         #[ink(constructor)]
///         pub fn new_storage_name() -> Contract {
///             Contract {}
///         }
///
///         #[ink(constructor)]
///         pub fn new_result_self() -> Result<Self, Error> {
///             Ok(Self {})
///         }
///
///         #[ink(constructor)]
///         pub fn new_result_storage_name() -> Result<Contract, Error> {
///             Ok(Contract {})
///         }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
/// ```
///
/// These constructor return signatures are then used by the `ContractRef` codegen for the
/// [`CreateBuilder::returns`] type parameter.
pub trait ConstructorReturnType<C> {
    /// Is `true` if `Self` is `Result<C, E>`.
    const IS_RESULT: bool = false;

    /// The actual return type of the constructor.
    /// - If a constructor returns `Self`, then `Output = Self`
    /// - If a constructor returns a `Result<Self, E>`, then `Output = Result<Self, E>`
    type Output;

    /// The error type of the constructor return type.
    type Error: scale::Decode;

    /// Construct a success value of the `Output` type.
    fn ok(value: C) -> Self::Output;

    /// Construct an error value of the `Output` type.
    ///
    /// `Result` implementations should return `Some(Err(err))`, otherwise default to
    /// `None`.
    fn err(_err: Self::Error) -> Option<Self::Output> {
        None
    }
}

/// Blanket implementation for `ContractRef` types, generated for cross-contract calls.
///
/// In the context of a `ContractRef` inherent, `Self` from a constructor return
/// type will become the type of the `ContractRef`'s type.
impl<C> ConstructorReturnType<C> for C
where
    C: ContractEnv + FromAccountId<<C as ContractEnv>::Env>,
{
    type Output = C;
    type Error = ();

    fn ok(value: C) -> Self::Output {
        value
    }
}

/// Blanket implementation for a `Result<Self>` return type. `Self` in the context
/// of a `ContractRef` inherent becomes the `ContractRef`s type.
impl<C, E> ConstructorReturnType<C> for core::result::Result<C, E>
where
    C: ContractEnv + FromAccountId<<C as ContractEnv>::Env>,
    E: scale::Decode,
{
    const IS_RESULT: bool = true;

    type Output = core::result::Result<C, E>;
    type Error = E;

    fn ok(value: C) -> Self::Output {
        Ok(value)
    }

    fn err(err: Self::Error) -> Option<Self::Output> {
        Some(Err(err))
    }
}

/// Defines the limit params for the legacy `ext::instantiate_v1` host function,
/// consisting of the `gas_limit` which is equivalent to the `ref_time_limit` in the new
/// `ext::instantiate`.
#[derive(Clone, Debug)]
pub struct LimitParamsV1 {
    gas_limit: u64,
}

/// Defines the limit params for the new `ext::instantiate` host function.
#[derive(Clone, Debug)]
pub struct LimitParamsV2<E>
where
    E: Environment,
{
    ref_time_limit: u64,
    proof_size_limit: u64,
    storage_deposit_limit: Option<E::Balance>,
}

/// Builds up contract instantiations.
#[derive(Debug)]
pub struct CreateParams<E, ContractRef, Limits, Args, Salt, R>
where
    E: Environment,
{
    /// The code hash of the created contract.
    code_hash: E::Hash,
    /// Parameters for weight and storage limits, differs for versions of the instantiate
    /// host function.
    limits: Limits,
    /// The endowment for the instantiated contract.
    endowment: E::Balance,
    /// The input data for the instantiation.
    exec_input: ExecutionInput<Args>,
    /// The salt for determining the hash for the contract account ID.
    salt_bytes: Salt,
    /// The return type of the target contract's constructor method.
    _return_type: ReturnType<R>,
    /// The type of the reference to the contract returned from the constructor.
    _phantom: PhantomData<fn() -> ContractRef>,
}

impl<E, ContractRef, Limits, Args, Salt, R>
    CreateParams<E, ContractRef, Limits, Args, Salt, R>
where
    E: Environment,
{
    /// The code hash of the contract.
    #[inline]
    pub fn code_hash(&self) -> &E::Hash {
        &self.code_hash
    }

    /// The endowment for the instantiated contract.
    #[inline]
    pub fn endowment(&self) -> &E::Balance {
        &self.endowment
    }

    /// The raw encoded input data.
    #[inline]
    pub fn exec_input(&self) -> &ExecutionInput<Args> {
        &self.exec_input
    }

    /// Modify the selector.
    ///
    /// Useful when using the [`CreateParams`] generated as part of the
    /// `ContractRef`, but using a custom selector.
    pub fn update_selector(&mut self, selector: Selector) {
        self.exec_input.update_selector(selector)
    }
}

impl<E, ContractRef, Args, Salt, R>
    CreateParams<E, ContractRef, LimitParamsV2<E>, Args, Salt, R>
where
    E: Environment,
{
    /// Gets the `ref_time_limit` part of the weight limit for the contract instantiation.
    #[inline]
    pub fn ref_time_limit(&self) -> u64 {
        self.limits.ref_time_limit
    }

    /// Gets the `proof_size_limit` part of the weight limit for the contract
    /// instantiation.
    #[inline]
    pub fn proof_size_limit(&self) -> u64 {
        self.limits.proof_size_limit
    }

    /// Gets the `storage_deposit_limit` for the contract instantiation.
    #[inline]
    pub fn storage_deposit_limit(&self) -> Option<&E::Balance> {
        self.limits.storage_deposit_limit.as_ref()
    }
}

impl<E, ContractRef, Args, Salt, R>
    CreateParams<E, ContractRef, LimitParamsV1, Args, Salt, R>
where
    E: Environment,
{
    /// The gas limit for the contract instantiation.
    #[inline]
    pub fn gas_limit(&self) -> u64 {
        self.limits.gas_limit
    }
}

impl<E, ContractRef, Limits, Args, Salt, R>
    CreateParams<E, ContractRef, Limits, Args, Salt, R>
where
    E: Environment,
    Salt: AsRef<[u8]>,
{
    /// The salt for determining the hash for the contract account ID.
    #[inline]
    pub fn salt_bytes(&self) -> &Salt {
        &self.salt_bytes
    }
}

impl<E, ContractRef, Args, Salt, R>
    CreateParams<E, ContractRef, LimitParamsV2<E>, Args, Salt, R>
where
    E: Environment,
    ContractRef: FromAccountId<E>,
    Args: scale::Encode,
    Salt: AsRef<[u8]>,
    R: ConstructorReturnType<ContractRef>,
{
    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_instantiate`][`CreateParams::try_instantiate`] method
    /// instead.
    #[inline]
    pub fn instantiate(&self) -> <R as ConstructorReturnType<ContractRef>>::Output {
        crate::instantiate_contract(self)
            .unwrap_or_else(|env_error| {
                panic!("Cross-contract instantiation failed with {env_error:?}")
            })
            .unwrap_or_else(|lang_error| {
                panic!("Received a `LangError` while instantiating: {lang_error:?}")
            })
    }

    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Note
    ///
    /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
    /// handled by the caller.
    #[inline]
    pub fn try_instantiate(
        &self,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <R as ConstructorReturnType<ContractRef>>::Output,
        >,
        Error,
    > {
        crate::instantiate_contract(self)
    }
}

// impl<E, ContractRef, Args, Salt, R>
//     CreateParams<E, ContractRef, LimitParamsV1, Args, Salt, R>
// where
//     E: Environment,
//     ContractRef: FromAccountId<E>,
//     Args: scale::Encode,
//     Salt: AsRef<[u8]>,
//     R: ConstructorReturnType<ContractRef>,
// {
//     /// Instantiates the contract and returns its account ID back to the caller.
//     ///
//     /// # Panics
//     ///
//     /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
//     /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
//     /// those use the [`try_instantiate`][`CreateParams::try_instantiate`] method
//     /// instead.
//     #[inline]
//     pub fn instantiate(&self) -> <R as ConstructorReturnType<ContractRef>>::Output {
//         self.try_instantiate()
//             .unwrap_or_else(|env_error| {
//                 panic!("Cross-contract instantiation failed with {env_error:?}")
//             })
//             .unwrap_or_else(|lang_error| {
//                 panic!("Received a `LangError` while instantiating: {lang_error:?}")
//             })
//     }
//
//     /// Instantiates the contract and returns its account ID back to the caller.
//     ///
//     /// # Note
//     ///
//     /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
//     /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
//     /// handled by the caller.
//     #[inline]
//     pub fn try_instantiate(
//         &self,
//     ) -> Result<
//         ink_primitives::ConstructorResult<
//             <R as ConstructorReturnType<ContractRef>>::Output,
//         >,
//         Error,
//     > {
//         crate::instantiate_contract_v1(self)
//     }
// }

/// Builds up contract instantiations.
#[derive(Clone)]
pub struct CreateBuilder<E, ContractRef, CodeHash, Limits, Endowment, Args, Salt, RetType>
where
    E: Environment,
{
    code_hash: CodeHash,
    limits: Limits,
    endowment: Endowment,
    exec_input: Args,
    salt: Salt,
    return_type: RetType,
    _phantom: PhantomData<fn() -> (E, ContractRef)>,
}

/// Returns a new [`CreateBuilder`] to build up the parameters to a cross-contract
/// instantiation.
///
/// # Example
///
/// **Note:** The shown examples panic because there is currently no cross-calling
///           support in the off-chain testing environment. However, this code
///           should work fine in on-chain environments.
///
/// ## Example 1: Returns Address of Instantiated Contract
///
/// The below example shows instantiation of contract of type `MyContract`.
///
/// The used constructor:
///
/// - has a selector equal to `0xDEADBEEF`
/// - is provided with 4000 units of gas for its execution
/// - is provided with 25 units of transferred value for the new contract instance
/// - receives the following arguments in order 1. an `i32` with value `42` 2. a `bool`
///   with value `true` 3. an array of 32 `u8` with value `0x10`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_create, Selector, ExecutionInput, FromAccountId}
/// # };
/// # type Hash = <DefaultEnvironment as Environment>::Hash;
/// #
/// # #[ink::contract]
/// # pub mod contract {
/// #     #[ink(storage)]
/// #     pub struct MyContract {}
/// #
/// #     impl MyContract {
/// #         #[ink(constructor)]
/// #         pub fn my_constructor() -> Self { Self {} }
/// #
/// #         #[ink(message)]
/// #         pub fn message(&self) {}
/// #     }
/// # }
/// # use contract::MyContractRef;
/// let my_contract: MyContractRef = build_create::<MyContractRef>()
///     .code_hash(Hash::from([0x42; 32]))
///     .endowment(25)
///     .exec_input(
///         ExecutionInput::new(Selector::new(ink::selector_bytes!("my_constructor")))
///             .push_arg(42)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32]),
///     )
///     .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
///     .returns::<MyContractRef>()
///     .instantiate();
/// ```
///
/// ## Example 2: Handles Result from Fallible Constructor
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_create, Selector, ExecutionInput, FromAccountId}
/// # };
/// # type Hash = <DefaultEnvironment as Environment>::Hash;
/// #
/// # #[ink::contract]
/// # pub mod contract {
/// #     #[derive(scale::Encode, scale::Decode, Debug)]
/// #     #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
/// #     pub struct ConstructorError;
/// #
/// #     #[ink(storage)]
/// #     pub struct MyContract {}
/// #
/// #     impl MyContract {
/// #         #[ink(constructor)]
/// #         pub fn my_constructor() -> Result<Self, ConstructorError> {
/// #             Ok(Self {})
/// #         }
/// #
/// #         #[ink(message)]
/// #         pub fn message(&self) {}
/// #     }
/// # }
/// # use contract::{MyContractRef, ConstructorError};
/// let my_contract: MyContractRef = build_create::<MyContractRef>()
///     .code_hash(Hash::from([0x42; 32]))
///     .endowment(25)
///     .exec_input(
///         ExecutionInput::new(Selector::new(ink::selector_bytes!("my_constructor")))
///             .push_arg(42)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32]),
///     )
///     .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
///     .returns::<Result<MyContractRef, ConstructorError>>()
///     .instantiate()
///     .expect("Constructor should have executed successfully.");
/// ```
#[allow(clippy::type_complexity)]
pub fn build_create<ContractRef>() -> CreateBuilder<
    <ContractRef as ContractEnv>::Env,
    ContractRef,
    Unset<<<ContractRef as ContractEnv>::Env as Environment>::Hash>,
    Set<LimitParamsV2<<ContractRef as ContractEnv>::Env>>,
    Unset<<<ContractRef as ContractEnv>::Env as Environment>::Balance>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    Unset<state::Salt>,
    Unset<ReturnType<()>>,
>
where
    ContractRef: ContractEnv,
{
    CreateBuilder {
        code_hash: Default::default(),
        limits: Set(LimitParamsV2 {
            ref_time_limit: 0,
            proof_size_limit: 0,
            storage_deposit_limit: None,
        }),
        endowment: Default::default(),
        exec_input: Default::default(),
        salt: Default::default(),
        return_type: Default::default(),
        _phantom: Default::default(),
    }
}

impl<E, ContractRef, Limits, Endowment, Args, Salt, RetType>
    CreateBuilder<E, ContractRef, Unset<E::Hash>, Limits, Endowment, Args, Salt, RetType>
where
    E: Environment,
{
    /// Sets the used code hash for the contract instantiation.
    #[inline]
    pub fn code_hash(
        self,
        code_hash: E::Hash,
    ) -> CreateBuilder<E, ContractRef, Set<E::Hash>, Limits, Endowment, Args, Salt, RetType>
    {
        CreateBuilder {
            code_hash: Set(code_hash),
            limits: self.limits,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, CodeHash, Endowment, Args, Salt, RetType>
    CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Set<LimitParamsV1>,
        Endowment,
        Args,
        Salt,
        RetType,
    >
where
    E: Environment,
{
    /// Sets the maximum allowed gas costs for the contract instantiation.
    #[inline]
    pub fn gas_limit(self, gas_limit: u64) -> Self {
        CreateBuilder {
            limits: Set(LimitParamsV1 { gas_limit }),
            ..self
        }
    }
}

impl<E, ContractRef, CodeHash, Endowment, Args, Salt, RetType>
    CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Set<LimitParamsV2<E>>,
        Endowment,
        Args,
        Salt,
        RetType,
    >
where
    E: Environment,
{
    /// Switch to the original `instantiate` host function API, which only allows the
    /// `gas_limit` limit parameter (equivalent to the `ref_time_limit` in the latest
    /// `instantiate_v2`).
    ///
    /// This method instance is used to allow usage of the generated builder methods
    /// for constructors which initialize the builder with the new [`LimitParamsV2`] type.
    #[inline]
    pub fn instantiate_v1(
        self,
    ) -> CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Set<LimitParamsV1>,
        Endowment,
        Args,
        Salt,
        RetType,
    > {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: Set(LimitParamsV1 {
                gas_limit: self.limits.value().ref_time_limit,
            }),
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }

    /// Sets the `ref_time_limit` part of the weight limit for the contract instantiation.
    #[inline]
    pub fn ref_time_limit(self, ref_time_limit: u64) -> Self {
        CreateBuilder {
            limits: Set(LimitParamsV2 {
                ref_time_limit,
                ..self.limits.value()
            }),
            ..self
        }
    }

    /// Sets the `proof_size_limit` part of the weight limit for the contract
    /// instantiation.
    #[inline]
    pub fn proof_size_limit(self, proof_size_limit: u64) -> Self {
        CreateBuilder {
            limits: Set(LimitParamsV2 {
                proof_size_limit,
                ..self.limits.value()
            }),
            ..self
        }
    }

    /// Sets the `storage_deposit_limit` for the contract instantiation.
    #[inline]
    pub fn storage_deposit_limit(self, storage_deposit_limit: E::Balance) -> Self {
        CreateBuilder {
            limits: Set(LimitParamsV2 {
                storage_deposit_limit: Some(storage_deposit_limit),
                ..self.limits.value()
            }),
            ..self
        }
    }
}

impl<E, ContractRef, CodeHash, Limits, Args, Salt, RetType>
    CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Unset<E::Balance>,
        Args,
        Salt,
        RetType,
    >
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn endowment(
        self,
        endowment: E::Balance,
    ) -> CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Set<E::Balance>,
        Args,
        Salt,
        RetType,
    > {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: self.limits,
            endowment: Set(endowment),
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, CodeHash, Limits, Endowment, Salt, RetType>
    CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Endowment,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Salt,
        RetType,
    >
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args>,
    ) -> CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Endowment,
        Set<ExecutionInput<Args>>,
        Salt,
        RetType,
    > {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: self.limits,
            endowment: self.endowment,
            exec_input: Set(exec_input),
            salt: self.salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, CodeHash, Limits, Endowment, Args, RetType>
    CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Endowment,
        Args,
        Unset<state::Salt>,
        RetType,
    >
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn salt_bytes<Salt>(
        self,
        salt: Salt,
    ) -> CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Endowment,
        Args,
        Set<Salt>,
        RetType,
    >
    where
        Salt: AsRef<[u8]>,
    {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: self.limits,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: Set(salt),
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, CodeHash, Limits, Endowment, Args, Salt>
    CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Endowment,
        Args,
        Salt,
        Unset<ReturnType<()>>,
    >
where
    E: Environment,
{
    /// Sets the type of the returned value upon the execution of the constructor.
    ///
    /// # Note
    ///
    /// Constructors are not able to return arbitrary values. Instead a successful call to
    /// a constructor returns the address at which the contract was instantiated.
    ///
    /// Therefore this must always be a reference (i.e `ContractRef`) to the contract
    /// you're trying to instantiate.
    #[inline]
    pub fn returns<R>(
        self,
    ) -> CreateBuilder<
        E,
        ContractRef,
        CodeHash,
        Limits,
        Endowment,
        Args,
        Salt,
        Set<ReturnType<R>>,
    >
    where
        ContractRef: FromAccountId<E>,
        R: ConstructorReturnType<ContractRef>,
    {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: self.limits,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: Set(Default::default()),
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, Limits, Args, Salt, RetType>
    CreateBuilder<
        E,
        ContractRef,
        Set<E::Hash>,
        Set<Limits>,
        Set<E::Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        Set<ReturnType<RetType>>,
    >
where
    E: Environment,
{
    /// Finalizes the create builder, allowing it to instantiate a contract.
    #[inline]
    pub fn params(self) -> CreateParams<E, ContractRef, Limits, Args, Salt, RetType> {
        CreateParams {
            code_hash: self.code_hash.value(),
            limits: self.limits.value(),
            endowment: self.endowment.value(),
            exec_input: self.exec_input.value(),
            salt_bytes: self.salt.value(),
            _return_type: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, Args, Salt, RetType>
    CreateBuilder<
        E,
        ContractRef,
        Set<E::Hash>,
        Set<LimitParamsV2<E>>,
        Set<E::Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        Set<ReturnType<RetType>>,
    >
where
    E: Environment,
    ContractRef: FromAccountId<E>,
    Args: scale::Encode,
    Salt: AsRef<[u8]>,
    RetType: ConstructorReturnType<ContractRef>,
{
    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_instantiate`][`CreateBuilder::try_instantiate`] method
    /// instead.
    #[inline]
    pub fn instantiate(self) -> <RetType as ConstructorReturnType<ContractRef>>::Output {
        self.params().instantiate()
    }

    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Note
    ///
    /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
    /// handled by the caller.
    #[inline]
    pub fn try_instantiate(
        self,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <RetType as ConstructorReturnType<ContractRef>>::Output,
        >,
        Error,
    > {
        self.params().try_instantiate()
    }
}

// impl<E, ContractRef, Args, Salt, RetType>
//     CreateBuilder<
//         E,
//         ContractRef,
//         Set<E::Hash>,
//         Set<LimitParamsV1>,
//         Set<E::Balance>,
//         Set<ExecutionInput<Args>>,
//         Set<Salt>,
//         Set<ReturnType<RetType>>,
//     >
// where
//     E: Environment,
//     ContractRef: FromAccountId<E>,
//     Args: scale::Encode,
//     Salt: AsRef<[u8]>,
//     RetType: ConstructorReturnType<ContractRef>,
// {
//     /// Instantiates the contract and returns its account ID back to the caller.
//     ///
//     /// # Panics
//     ///
//     /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
//     /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
//     /// those use the [`try_instantiate`][`CreateBuilder::try_instantiate`] method
//     /// instead.
//     #[inline]
//     pub fn instantiate(self) -> <RetType as ConstructorReturnType<ContractRef>>::Output {
//         self.params().instantiate()
//     }
//
//     /// Instantiates the contract and returns its account ID back to the caller.
//     ///
//     /// # Note
//     ///
//     /// On failure this returns an outer [`ink::env::Error`][`crate::Error`] or inner
//     /// [`ink::primitives::LangError`][`ink_primitives::LangError`], both of which can be
//     /// handled by the caller.
//     #[inline]
//     pub fn try_instantiate(
//         self,
//     ) -> Result<
//         ink_primitives::ConstructorResult<
//             <RetType as ConstructorReturnType<ContractRef>>::Output,
//         >,
//         Error,
//     > {
//         self.params().try_instantiate()
//     }
// }
