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

use core::marker::PhantomData;

use ink_primitives::{
    Address,
    H256,
    U256,
    abi::{
        AbiEncodeWith,
        Ink,
        Sol,
    },
};

use crate::{
    ContractEnv,
    Error,
    call::{
        ExecutionInput,
        Selector,
        utils::{
            DecodeConstructorError,
            EmptyArgumentList,
            ReturnType,
            Set,
            Unset,
        },
    },
    types::Environment,
};

pub mod state {
    //! Type states that tell what state of a instantiation argument has not
    //! yet been set properly for a valid construction.
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
pub trait FromAddr {
    /// Creates the contract instance from the account ID of the already instantiated
    /// contract.
    fn from_addr(addr: Address) -> Self;
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
pub trait ConstructorReturnType<C, Abi> {
    /// Is `true` if `Self` is `Result<C, E>`.
    const IS_RESULT: bool = false;

    /// The actual return type of the constructor.
    /// - If a constructor returns `Self`, then `Output = Self`
    /// - If a constructor returns a `Result<Self, E>`, then `Output = Result<Self, E>`
    type Output;

    /// The error type of the constructor return type.
    type Error: DecodeConstructorError<Abi>;

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
impl<C, Abi> ConstructorReturnType<C, Abi> for C
where
    C: ContractEnv + FromAddr,
    (): DecodeConstructorError<Abi>,
{
    type Output = C;
    type Error = ();

    fn ok(value: C) -> Self::Output {
        value
    }
}

/// Blanket implementation for a `Result<Self>` return type. `Self` in the context
/// of a `ContractRef` inherent becomes the `ContractRef`s type.
impl<C, E, Abi> ConstructorReturnType<C, Abi> for Result<C, E>
where
    C: ContractEnv + FromAddr,
    E: DecodeConstructorError<Abi>,
{
    const IS_RESULT: bool = true;

    type Output = Result<C, E>;
    type Error = E;

    fn ok(value: C) -> Self::Output {
        Ok(value)
    }

    fn err(err: Self::Error) -> Option<Self::Output> {
        Some(Err(err))
    }
}

/// Defines the limit params for the new `ext::instantiate` host function.
/// todo: rename
#[derive(Clone, Debug)]
pub struct LimitParamsV2 {
    ref_time_limit: u64,
    proof_size_limit: u64,
    storage_deposit_limit: Option<U256>,
}

/// Builds up contract instantiations.
#[derive(Debug)]
pub struct CreateParams<E, ContractRef, Limits, Args, R, Abi> {
    /// The code hash of the created contract.
    code_hash: H256,
    /// Parameters for weight and storage limits, differs for versions of the instantiate
    /// host function.
    limits: Limits,
    /// The endowment for the instantiated contract.
    /// todo: is this correct? or is the value here `U256`?
    endowment: U256,
    /// The input data for the instantiation.
    exec_input: ExecutionInput<Args, Abi>,
    /// The salt for determining the hash for the contract account ID.
    salt_bytes: Option<[u8; 32]>,
    /// The return type of the target contract's constructor method.
    _return_type: ReturnType<R>,
    /// The type of the reference to the contract returned from the constructor.
    _phantom: PhantomData<fn() -> (E, ContractRef)>,
}

impl<E, ContractRef, Limits, Args, R, Abi>
    CreateParams<E, ContractRef, Limits, Args, R, Abi>
where
    E: Environment,
{
    /// The code hash of the contract.
    #[inline]
    pub fn code_hash(&self) -> &H256 {
        &self.code_hash
    }

    /// The endowment for the instantiated contract.
    #[inline]
    pub fn endowment(&self) -> &U256 {
        &self.endowment
    }

    /// The raw encoded input data.
    #[inline]
    pub fn exec_input(&self) -> &ExecutionInput<Args, Abi> {
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

impl<E, ContractRef, Args, R, Abi>
    CreateParams<E, ContractRef, LimitParamsV2, Args, R, Abi>
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
    pub fn storage_deposit_limit(&self) -> Option<&U256> {
        self.limits.storage_deposit_limit.as_ref()
    }
}

impl<E, ContractRef, Limits, Args, R, Abi>
    CreateParams<E, ContractRef, Limits, Args, R, Abi>
where
    E: Environment,
{
    /// The salt for determining the hash for the contract account ID.
    #[inline]
    pub fn salt_bytes(&self) -> &Option<[u8; 32]> {
        &self.salt_bytes
    }
}

impl<E, ContractRef, Args, R, Abi>
    CreateParams<E, ContractRef, LimitParamsV2, Args, R, Abi>
where
    E: Environment,
    ContractRef: FromAddr + crate::ContractReverseReference,
    <ContractRef as crate::ContractReverseReference>::Type:
        crate::reflect::ContractConstructorDecoder,
    <ContractRef as crate::ContractReverseReference>::Type:
        crate::reflect::ContractMessageDecoder,
    Args: AbiEncodeWith<Abi>,
    R: ConstructorReturnType<ContractRef, Abi>,
{
    /// todo
    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_instantiate`][`CreateParams::try_instantiate`] method
    /// instead.
    #[inline]
    pub fn instantiate(&self) -> <R as ConstructorReturnType<ContractRef, Abi>>::Output {
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
            <R as ConstructorReturnType<ContractRef, Abi>>::Output,
        >,
        Error,
    > {
        crate::instantiate_contract(self)
    }
}

/// Builds up contract instantiations.
#[derive(Clone)]
pub struct CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi>
where
    E: Environment,
{
    code_hash: H256,
    limits: Limits,
    endowment: U256,
    exec_input: Args,
    salt: Option<[u8; 32]>,
    return_type: RetType,
    #[allow(clippy::type_complexity)]
    _phantom: PhantomData<fn() -> (E, ContractRef, Abi)>,
}

/// Returns a new [`CreateBuilder`] to build up the parameters to a cross-contract
/// instantiation that uses the "default" ABI for calls for the ink! project.
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
/// #     call::{build_create, Selector, ExecutionInput, FromAddr}
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
///     .code_hash(ink::H256::from([0x42; 32]))
///     .endowment(25.into())
///     .exec_input(
///         ExecutionInput::new(Selector::new(ink::selector_bytes!("my_constructor")))
///             .push_arg(42)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32]),
///     )
///     .salt_bytes(Some([1u8; 32]))
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
/// #     call::{build_create, Selector, ExecutionInput, FromAddr}
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
///     .code_hash(ink::H256::from([0x42; 32]))
///     .endowment(25.into())
///     .exec_input(
///         ExecutionInput::new(Selector::new(ink::selector_bytes!("my_constructor")))
///             .push_arg(42)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32]),
///     )
///     .salt_bytes(Some([1u8; 32]))
///     .returns::<Result<MyContractRef, ConstructorError>>()
///     .instantiate()
///     .expect("Constructor should have executed successfully.");
/// ```
#[allow(clippy::type_complexity)]
pub fn build_create<ContractRef>() -> CreateBuilder<
    <ContractRef as ContractEnv>::Env,
    ContractRef,
    Set<LimitParamsV2>,
    Unset<ExecutionInput<EmptyArgumentList<crate::DefaultAbi>, crate::DefaultAbi>>,
    Unset<ReturnType<()>>,
    crate::DefaultAbi,
>
where
    ContractRef: ContractEnv,
{
    CreateBuilder {
        code_hash: Default::default(),
        limits: Set(LimitParamsV2 {
            ref_time_limit: u64::MAX,
            proof_size_limit: u64::MAX,
            storage_deposit_limit: None,
        }),
        endowment: Default::default(),
        exec_input: Default::default(),
        salt: Default::default(),
        return_type: Default::default(),
        _phantom: Default::default(),
    }
}

/// Returns a new [`CreateBuilder`] to build up the parameters to a cross-contract
/// instantiation that uses ink! ABI Encoding (i.e. with SCALE codec for input/output
/// encode/decode).
///
/// See [`build_create`] for more details on usage.
#[allow(clippy::type_complexity)]
pub fn build_create_ink<ContractRef>() -> CreateBuilder<
    <ContractRef as ContractEnv>::Env,
    ContractRef,
    Set<LimitParamsV2>,
    Unset<ExecutionInput<EmptyArgumentList<Ink>, Ink>>,
    Unset<ReturnType<()>>,
    Ink,
>
where
    ContractRef: ContractEnv,
{
    CreateBuilder {
        code_hash: Default::default(),
        limits: Set(LimitParamsV2 {
            ref_time_limit: u64::MAX,
            proof_size_limit: u64::MAX,
            storage_deposit_limit: None,
        }),
        endowment: Default::default(),
        exec_input: Default::default(),
        salt: Default::default(),
        return_type: Default::default(),
        _phantom: Default::default(),
    }
}

/// Returns a new [`CreateBuilder`] to build up the parameters to a cross-contract
/// instantiation that uses Solidity ABI Encoding.
///
/// See [`build_create`] for more details on usage.
#[allow(clippy::type_complexity)]
pub fn build_create_sol<ContractRef>() -> CreateBuilder<
    <ContractRef as ContractEnv>::Env,
    ContractRef,
    Set<LimitParamsV2>,
    Unset<ExecutionInput<EmptyArgumentList<Sol>, Sol>>,
    Unset<ReturnType<()>>,
    Sol,
>
where
    ContractRef: ContractEnv,
{
    CreateBuilder {
        code_hash: Default::default(),
        limits: Set(LimitParamsV2 {
            ref_time_limit: u64::MAX,
            proof_size_limit: u64::MAX,
            storage_deposit_limit: None,
        }),
        endowment: Default::default(),
        exec_input: Default::default(),
        salt: Default::default(),
        return_type: Default::default(),
        _phantom: Default::default(),
    }
}

impl<E, ContractRef, Limits, Args, RetType, Abi>
    CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi>
where
    E: Environment,
{
    /// Sets the used code hash for the contract instantiation.
    #[inline]
    pub fn code_hash(
        self,
        code_hash: H256,
    ) -> CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi> {
        CreateBuilder {
            code_hash,
            limits: self.limits,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, Args, RetType, Abi>
    CreateBuilder<E, ContractRef, Set<LimitParamsV2>, Args, RetType, Abi>
where
    E: Environment,
{
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
    pub fn storage_deposit_limit(self, storage_deposit_limit: U256) -> Self {
        CreateBuilder {
            limits: Set(LimitParamsV2 {
                storage_deposit_limit: Some(storage_deposit_limit),
                ..self.limits.value()
            }),
            ..self
        }
    }
}

impl<E, ContractRef, Limits, Args, RetType, Abi>
    CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi>
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn endowment(
        self,
        endowment: U256,
    ) -> CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi> {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: self.limits,
            endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, Limits, RetType, Abi>
    CreateBuilder<
        E,
        ContractRef,
        Limits,
        Unset<ExecutionInput<EmptyArgumentList<Abi>, Abi>>,
        RetType,
        Abi,
    >
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args, Abi>,
    ) -> CreateBuilder<E, ContractRef, Limits, Set<ExecutionInput<Args, Abi>>, RetType, Abi>
    {
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

impl<E, ContractRef, Limits, Args, RetType, Abi>
    CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi>
where
    E: Environment,
{
    /// Sets the salt used for the execution of the call.
    #[inline]
    pub fn salt_bytes(
        self,
        salt: Option<[u8; 32]>,
    ) -> CreateBuilder<E, ContractRef, Limits, Args, RetType, Abi> {
        CreateBuilder {
            code_hash: self.code_hash,
            limits: self.limits,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt,
            return_type: self.return_type,
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, Limits, Args, Abi>
    CreateBuilder<E, ContractRef, Limits, Args, Unset<ReturnType<()>>, Abi>
where
    E: Environment,
{
    /// Sets the type of the returned value upon the execution of the constructor.
    ///
    /// # Note
    ///
    /// Constructors are not able to return arbitrary values. Instead, a successful call
    /// to a constructor returns the address at which the contract was instantiated.
    ///
    /// Therefore this must always be a reference (i.e. `ContractRef`) to the contract
    /// you're trying to instantiate.
    #[inline]
    pub fn returns<R>(
        self,
    ) -> CreateBuilder<E, ContractRef, Limits, Args, Set<ReturnType<R>>, Abi>
    where
        ContractRef: FromAddr,
        R: ConstructorReturnType<ContractRef, Abi>,
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

impl<E, ContractRef, Limits, Args, RetType, Abi>
    CreateBuilder<
        E,
        ContractRef,
        Set<Limits>,
        Set<ExecutionInput<Args, Abi>>,
        Set<ReturnType<RetType>>,
        Abi,
    >
where
    E: Environment,
{
    /// Finalizes the `CreateBuilder`, allowing it to instantiate a contract.
    #[inline]
    pub fn params(self) -> CreateParams<E, ContractRef, Limits, Args, RetType, Abi> {
        CreateParams {
            code_hash: self.code_hash,
            limits: self.limits.value(),
            endowment: self.endowment,
            exec_input: self.exec_input.value(),
            salt_bytes: self.salt,
            _return_type: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<E, ContractRef, Args, RetType, Abi>
    CreateBuilder<
        E,
        ContractRef,
        Set<LimitParamsV2>,
        Set<ExecutionInput<Args, Abi>>,
        Set<ReturnType<RetType>>,
        Abi,
    >
where
    E: Environment,
    ContractRef: FromAddr + crate::ContractReverseReference,
    <ContractRef as crate::ContractReverseReference>::Type:
        crate::reflect::ContractConstructorDecoder,
    <ContractRef as crate::ContractReverseReference>::Type:
        crate::reflect::ContractMessageDecoder,
    Args: AbiEncodeWith<Abi>,
    RetType: ConstructorReturnType<ContractRef, Abi>,
{
    /// todo check comment
    /// Instantiates the contract and returns its account ID back to the caller.
    ///
    /// # Panics
    ///
    /// This method panics if it encounters an [`ink::env::Error`][`crate::Error`] or an
    /// [`ink::primitives::LangError`][`ink_primitives::LangError`]. If you want to handle
    /// those use the [`try_instantiate`][`CreateBuilder::try_instantiate`] method
    /// instead.
    #[inline]
    pub fn instantiate(
        self,
    ) -> <RetType as ConstructorReturnType<ContractRef, Abi>>::Output {
        self.params().instantiate()
    }

    /// todo check comment
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
            <RetType as ConstructorReturnType<ContractRef, Abi>>::Output,
        >,
        Error,
    > {
        self.params().try_instantiate()
    }
}
