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
/// in the generated code of `ink_lang`.
pub trait FromAccountId<T>
where
    T: Environment,
{
    /// Creates the contract instance from the account ID of the already instantiated contract.
    fn from_account_id(account_id: <T as Environment>::AccountId) -> Self;
}

/// Builds up contract instantiations.
#[derive(Debug)]
pub struct CreateParams<E, Args, Salt, R>
where
    E: Environment,
{
    /// The code hash of the created contract.
    code_hash: E::Hash,
    /// The maximum gas costs allowed for the instantiation.
    gas_limit: u64,
    /// The endowment for the instantiated contract.
    endowment: E::Balance,
    /// The input data for the instantiation.
    exec_input: ExecutionInput<Args>,
    /// The salt for determining the hash for the contract account ID.
    salt_bytes: Salt,
    /// The type of the instantiated contract.
    return_type: ReturnType<R>,
}

cfg_if::cfg_if! {
    // We do not currently support cross-contract instantiation in the off-chain
    // environment so we do not have to provide these getters in case of
    // off-chain environment compilation.
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {
        impl<E, Args, Salt, R> CreateParams<E, Args, Salt, R>
        where
            E: Environment,
        {
            /// The code hash of the contract.
            #[inline]
            pub(crate) fn code_hash(&self) -> &E::Hash {
                &self.code_hash
            }

            /// The gas limit for the contract instantiation.
            #[inline]
            pub(crate) fn gas_limit(&self) -> u64 {
                self.gas_limit
            }

            /// The endowment for the instantiated contract.
            #[inline]
            pub(crate) fn endowment(&self) -> &E::Balance {
                &self.endowment
            }

            /// The raw encoded input data.
            #[inline]
            pub(crate) fn exec_input(&self) -> &ExecutionInput<Args> {
                &self.exec_input
            }
        }

        impl<E, Args, Salt, R> CreateParams<E, Args, Salt, R>
        where
            E: Environment,
            Salt: AsRef<[u8]>,
        {
            /// The salt for determining the hash for the contract account ID.
            #[inline]
            pub(crate) fn salt_bytes(&self) -> &Salt {
                &self.salt_bytes
            }
        }
    }
}

impl<E, Args, Salt, R> CreateParams<E, Args, Salt, R>
where
    E: Environment,
    Args: scale::Encode,
    Salt: AsRef<[u8]>,
    R: FromAccountId<E>,
{
    /// Instantiates the contract and returns its account ID back to the caller.
    #[inline]
    pub fn instantiate(&self) -> Result<R, crate::Error> {
        crate::instantiate_contract(self).map(FromAccountId::from_account_id)
    }
}

/// Builds up contract instantiations.
pub struct CreateBuilder<E, CodeHash, GasLimit, Endowment, Args, Salt, R>
where
    E: Environment,
{
    env: PhantomData<fn() -> E>,
    code_hash: CodeHash,
    gas_limit: GasLimit,
    endowment: Endowment,
    exec_input: Args,
    salt: Salt,
    return_type: ReturnType<R>,
}

/// Returns a new [`CreateBuilder`] to build up the parameters to a cross-contract instantiation.
///
/// # Example
///
/// The below example shows instantiation of contract of type `MyContract`.
///
/// The used constructor ...
///
/// - has a selector equal to `0xDEADBEEF`
/// - is provided with 4000 units of gas for its execution
/// - is provided with 25 units of transferred value for the new contract instance
/// - receives the following arguments in order
///    1. an `i32` with value `42`
///    2. a `bool` with value `true`
///    3. an array of 32 `u8` with value `0x10`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_create, Selector, ExecutionInput, FromAccountId}
/// # };
/// # type Hash = <DefaultEnvironment as Environment>::Hash;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Salt = &'static [u8];
/// # struct MyContract;
/// # impl FromAccountId<DefaultEnvironment> for MyContract {
/// #     fn from_account_id(account_id: AccountId) -> Self { Self }
/// # }
/// let my_contract: MyContract = build_create::<DefaultEnvironment, MyContract>()
///     .code_hash(Hash::from([0x42; 32]))
///     .gas_limit(4000)
///     .endowment(25)
///     .exec_input(
///         ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
///             .push_arg(42)
///             .push_arg(true)
///             .push_arg(&[0x10u8; 32])
///     )
///     .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
///     .params()
///     .instantiate()
///     .unwrap();
/// ```
///
/// **Note:** The shown example panics because there is currently no cross-calling
///           support in the off-chain testing environment. However, this code
///           should work fine in on-chain environments.
#[allow(clippy::type_complexity)]
pub fn build_create<E, R>() -> CreateBuilder<
    E,
    Unset<E::Hash>,
    Unset<u64>,
    Unset<E::Balance>,
    Unset<ExecutionInput<EmptyArgumentList>>,
    Unset<state::Salt>,
    R,
>
where
    E: Environment,
    R: FromAccountId<E>,
{
    CreateBuilder {
        env: Default::default(),
        code_hash: Default::default(),
        gas_limit: Default::default(),
        endowment: Default::default(),
        exec_input: Default::default(),
        salt: Default::default(),
        return_type: Default::default(),
    }
}

impl<E, GasLimit, Endowment, Args, Salt, R>
    CreateBuilder<E, Unset<E::Hash>, GasLimit, Endowment, Args, Salt, R>
where
    E: Environment,
{
    /// Sets the used code hash for the contract instantiation.
    #[inline]
    pub fn code_hash(
        self,
        code_hash: E::Hash,
    ) -> CreateBuilder<E, Set<E::Hash>, GasLimit, Endowment, Args, Salt, R> {
        CreateBuilder {
            env: Default::default(),
            code_hash: Set(code_hash),
            gas_limit: self.gas_limit,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, Endowment, Args, Salt, R>
    CreateBuilder<E, CodeHash, Unset<u64>, Endowment, Args, Salt, R>
where
    E: Environment,
{
    /// Sets the maximum allowed gas costs for the contract instantiation.
    #[inline]
    pub fn gas_limit(
        self,
        gas_limit: u64,
    ) -> CreateBuilder<E, CodeHash, Set<u64>, Endowment, Args, Salt, R> {
        CreateBuilder {
            env: Default::default(),
            code_hash: self.code_hash,
            gas_limit: Set(gas_limit),
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, GasLimit, Args, Salt, R>
    CreateBuilder<E, CodeHash, GasLimit, Unset<E::Balance>, Args, Salt, R>
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn endowment(
        self,
        endowment: E::Balance,
    ) -> CreateBuilder<E, CodeHash, GasLimit, Set<E::Balance>, Args, Salt, R> {
        CreateBuilder {
            env: Default::default(),
            code_hash: self.code_hash,
            gas_limit: self.gas_limit,
            endowment: Set(endowment),
            exec_input: self.exec_input,
            salt: self.salt,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, GasLimit, Endowment, Salt, R>
    CreateBuilder<
        E,
        CodeHash,
        GasLimit,
        Endowment,
        Unset<ExecutionInput<EmptyArgumentList>>,
        Salt,
        R,
    >
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn exec_input<Args>(
        self,
        exec_input: ExecutionInput<Args>,
    ) -> CreateBuilder<E, CodeHash, GasLimit, Endowment, Set<ExecutionInput<Args>>, Salt, R>
    {
        CreateBuilder {
            env: Default::default(),
            code_hash: self.code_hash,
            gas_limit: self.gas_limit,
            endowment: self.endowment,
            exec_input: Set(exec_input),
            salt: self.salt,
            return_type: self.return_type,
        }
    }
}

impl<E, CodeHash, GasLimit, Endowment, Args, R>
    CreateBuilder<E, CodeHash, GasLimit, Endowment, Args, Unset<state::Salt>, R>
where
    E: Environment,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn salt_bytes<Salt>(
        self,
        salt: Salt,
    ) -> CreateBuilder<E, CodeHash, GasLimit, Endowment, Args, Set<Salt>, R>
    where
        Salt: AsRef<[u8]>,
    {
        CreateBuilder {
            env: Default::default(),
            code_hash: self.code_hash,
            gas_limit: self.gas_limit,
            endowment: self.endowment,
            exec_input: self.exec_input,
            salt: Set(salt),
            return_type: self.return_type,
        }
    }
}

impl<E, GasLimit, Args, Salt, R>
    CreateBuilder<
        E,
        Set<E::Hash>,
        GasLimit,
        Set<E::Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        R,
    >
where
    E: Environment,
    GasLimit: Unwrap<Output = u64>,
{
    /// Sets the value transferred upon the execution of the call.
    #[inline]
    pub fn params(self) -> CreateParams<E, Args, Salt, R> {
        CreateParams {
            code_hash: self.code_hash.value(),
            gas_limit: self.gas_limit.unwrap_or_else(|| 0),
            endowment: self.endowment.value(),
            exec_input: self.exec_input.value(),
            salt_bytes: self.salt.value(),
            return_type: self.return_type,
        }
    }
}

impl<E, GasLimit, Args, Salt, R>
    CreateBuilder<
        E,
        Set<E::Hash>,
        GasLimit,
        Set<E::Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        R,
    >
where
    E: Environment,
    GasLimit: Unwrap<Output = u64>,
    Args: scale::Encode,
    Salt: AsRef<[u8]>,
    R: FromAccountId<E>,
{
    /// Instantiates the contract using the given instantiation parameters.
    #[inline]
    pub fn instantiate(self) -> Result<R, Error> {
        self.params().instantiate()
    }
}
