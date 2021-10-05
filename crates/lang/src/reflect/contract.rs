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

/// Stores the name of the ink! smart contract.
///
/// # Note
///
/// The name is the identifier of the `#[ink(storage)]` annotated `struct`.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// # use ink_lang::reflect::ContractName;
/// assert_eq!(
///     <Contract as ContractName>::NAME,
///     "Contract",
/// );
/// ```
pub trait ContractName {
    /// The name of the ink! smart contract.
    const NAME: &'static str;
}

/// Stores the used host environment type of the ink! smart contract.
///
/// # Note
///
/// The used host environment can be altered using the `env` configuration
/// parameter in the `#[ink::contract]` parameters. For example if the user
/// wanted to use an environment type definition called `MyEnvironment` they
/// issue the ink! smart contract as follows:
///
/// ```
/// #[ink::contract(env = MyEnvironment)]
/// ```
///
/// # Usage: Default Environment
///
/// ```
/// use ink_lang as ink;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// # use ink_lang::reflect::ContractEnv;
/// assert_eq!(
///     <Contract as ContractEnv>::Env,
///     ink_env::DefaultEnvironment,
/// );
/// ```
///
/// # Usage: Custom Environment
///
/// ```
/// use ink_lang as ink;
/// # use ink_env::Environment;
///
/// pub struct CustomEnvironment {}
///
/// impl Environment for CustomEnvironment {
///     const MAX_EVENT_TOPICS: usize = 4;
///
///     type AccountId = <DefaultEnvironment as Environment>::AccountId;
///     type Balance = u64;
///     type Hash = <DefaultEnvironment as Environment>::Hash;
///     type BlockNumber = u32;
///     type Timestamp = u64;
///     type RentFraction = <DefaultEnvironment as Environment>::RentFraction;
///     type ChainExtension = <DefaultEnvironment as Environment>::ChainExtension;
/// }
///
/// #[ink::contract(env = super::CustomEnvironment)]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// # use ink_lang::reflect::ContractEnv;
/// assert_eq!(
///     <Contract as ContractEnv>::Env,
///     CustomEnvironment
/// );
/// ```
pub trait ContractEnv {
    /// The environment type.
    type Env: ::ink_env::Environment;
}
