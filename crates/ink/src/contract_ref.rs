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

use ink_env::Environment;

/// The macro allows the creation of a wrapper around the trait defined with
/// [`crate::trait_definition`]. This wrapper can be used for interaction with
/// the contract.
///
/// The macro returns the native Rust type that implements the corresponding trait.
///
/// The macro expects two arguments:
/// - The first argument is the path of the trait, like `Erc20` or `erc20::Erc20`.
/// - The second argument is optional and specifies the type of the [`Environment`].
///   If the environment is not specified, the macro uses the [`crate::env::DefaultEnvironment`].
///
/// ```rust
/// #[ink::contract]
/// mod trait_caller {
///      use ink::contract_ref;
/// #    #[ink::trait_definition]
/// #    pub trait Erc20 {
/// #       /// Returns the total supply of the ERC-20 smart contract.
/// #       #[ink(message)]
/// #       fn total_supply(&self) -> Balance;
/// #
/// #       /// Transfers balance from the caller to the given address.
/// #       #[ink(message)]
/// #       fn transfer(&mut self, amount: Balance, to: AccountId) -> bool;
/// #    }
/// #
///     #[ink(storage)]
///     pub struct Caller {
///         /// The example of `contract_ref!` as a struct type.
///         erc20: contract_ref!(Erc20),
///     }
///
///     impl Caller {
///         /// The example of `contract_ref!` as an argument type.
///         #[ink(constructor)]
///         pub fn new(erc20: contract_ref!(Erc20)) -> Self {
///             Self { erc20 }
///         }
///
///         /// The example of converting `AccountId` into `contract_ref!` implicitly.
///         #[ink(message)]
///         pub fn change_account_id_1(&mut self, new_erc20: AccountId) {
///             self.erc20 = new_erc20.into();
///         }
///
///         /// The example of converting `AccountId` into `contract_ref!` explicitly.
///         #[ink(message)]
///         pub fn change_account_id_2(&mut self, new_erc20: AccountId) {
///             let erc20: contract_ref!(Erc20) = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// The example of converting `AccountId` into `contract_ref!` explicitly
///         /// with custom environment.
///         #[ink(message)]
///         pub fn change_account_id_3(&mut self, new_erc20: AccountId) {
///             let erc20: contract_ref!(Erc20, ink::env::DefaultEnvironment) = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// The example of converting `AccountId` into `contract_ref!` explicitly
///         /// with custom environment from the associated type.
///         #[ink(message)]
///         pub fn change_account_id_4(&mut self, new_erc20: AccountId) {
///             let erc20: contract_ref!(Erc20, <Caller as ink::env::ContractEnv>::Env) = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// The example of how to do common calls.
///         #[ink(message)]
///         pub fn total_supply_1(&self) -> Balance {
///             self.erc20.total_supply()
///         }
///
///         /// The example of how to use the call builder with `contract_ref!`.
///         #[ink(message)]
///         pub fn total_supply_2(&self) -> Balance {
///             use ink::codegen::TraitCallBuilder;
///             let erc20_builder = self.erc20.call();
///             let err: ink::env::Result<ink::primitives::MessageResult<Balance>> =
///                 erc20_builder.total_supply().transferred_value(0).try_invoke();
///             err
///                 .expect("The cross-contract call should be executed without ink::env::Error")
///                 .expect("The cross-contract call should be executed without ink::primitives::LangError")
///         }
///
///         /// The example of how to do common calls and convert the `contract_ref!` into `AccountId`.
///         #[ink(message)]
///         pub fn transfer_to_erc20(&mut self, amount: Balance) -> bool {
///             let erc20_as_account_id = self.erc20.as_ref().clone();
///             self.erc20.transfer(amount, erc20_as_account_id)
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! contract_ref {
    // The case of the default `Environment`
    ( $trait_path:path ) => {
        $crate::contract_ref!($trait_path, $crate::env::DefaultEnvironment)
    };
    // The case of the custom `Environment`
    ( $trait_path:path, $env:ty ) => {
        <<$crate::reflect::TraitDefinitionRegistry<$env> as $trait_path>
                            ::__ink_TraitInfo as $crate::codegen::TraitCallForwarder>
                                ::Forwarder
    };
}

/// Implemented by contracts that are compiled as dependencies.
///
/// Allows them to return their underlying account identifier.
pub trait ToAccountId<T>
where
    T: Environment,
{
    /// Returns the underlying account identifier of the instantiated contract.
    fn to_account_id(&self) -> <T as Environment>::AccountId;
}
