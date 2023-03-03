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

/// Generates a wrapper which can be used for interacting with the contract.
///
/// This is done by creating a wrapper around the trait defined with the
/// [`ink::trait_definition`](crate::trait_definition) macro.
///
/// The macro returns the native Rust type that implements the corresponding trait,
/// so it can be used in any Rust context that expects types.
///
/// # Usage in the `#[ink::contract]` context
///
/// The macro expects one argument ‒ the path to the trait, in order to create a wrapper around it.
///
/// ```rust
/// #[ink::contract]
/// mod trait_caller {
///     use ink::contract_ref;
///
///     #[ink::trait_definition]
///     pub trait Erc20 {
///        /// Returns the total supply of the ERC-20 smart contract.
///        #[ink(message)]
///        fn total_supply(&self) -> Balance;
///
///        /// Transfers balance from the caller to the given address.
///        #[ink(message)]
///        fn transfer(&mut self, amount: Balance, to: AccountId) -> bool;
///     }
///
///     #[ink(storage)]
///     pub struct Caller {
///         /// The example of `contract_ref!` as a struct type.
///         erc20: contract_ref!(Erc20),
///     }
///
///     impl Caller {
///         /// Example of `contract_ref!` as an argument type.
///         #[ink(constructor)]
///         pub fn new(erc20: contract_ref!(Erc20)) -> Self {
///             Self { erc20 }
///         }
///
///         /// Example of converting `AccountId` into `contract_ref!` implicitly.
///         #[ink(message)]
///         pub fn change_account_id_1(&mut self, new_erc20: AccountId) {
///             self.erc20 = new_erc20.into();
///         }
///
///         /// Example of converting `AccountId` into `contract_ref!` explicitly.
///         #[ink(message)]
///         pub fn change_account_id_2(&mut self, new_erc20: AccountId) {
///             let erc20: contract_ref!(Erc20) = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// Example of converting `AccountId` into an alias from `contract_ref!`.
///         #[ink(message)]
///         pub fn change_account_id_3(&mut self, new_erc20: AccountId) {
///             type Erc20Wrapper = contract_ref!(Erc20);
///             let erc20: Erc20Wrapper = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// Example of how to do common calls via fully qualified syntax.
///         #[ink(message)]
///         pub fn total_supply_1(&self) -> Balance {
///             Erc20::total_supply(&self.erc20)
///         }
///
///         /// Example of how to do common calls without fully qualified syntax.
///         #[ink(message)]
///         pub fn total_supply_2(&self) -> Balance {
///             self.erc20.total_supply()
///         }
///
///         /// Example of how to use the call builder with `contract_ref!`.
///         #[ink(message)]
///         pub fn total_supply_3(&self) -> Balance {
///             use ink::codegen::TraitCallBuilder;
///             // Returns the `CallBuilder` that implements `Erc20` trait.
///             let erc20_builder = self.erc20.call();
///             erc20_builder.total_supply().transferred_value(0).invoke()
///         }
///
///         /// Example of how to do common calls and convert
///         /// the `contract_ref!` into `AccountId`.
///         #[ink(message)]
///         pub fn transfer_to_erc20(&mut self, amount: Balance) -> bool {
///             let erc20_as_account_id = self.erc20.as_ref().clone();
///             self.erc20.transfer(amount, erc20_as_account_id)
///         }
///     }
/// }
/// ```
///
/// # Usage outside of the `#[ink::contract]` context
///
/// The macro expects two arguments:
/// - The first argument is the path to the trait, e.g. `Erc20` or `erc20::Erc20`.
/// - The second argument is the type of the [`ink_env::Environment`].
///
/// If the second argument is not specified, the macro uses the `Environment` type alias.
///
/// ```rust
/// use ink::contract_ref;
/// use ink_env::DefaultEnvironment;
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
/// pub struct CustomEnv;
///
/// impl ink_env::Environment for CustomEnv {
///     const MAX_EVENT_TOPICS: usize = 3;
///     type AccountId = [u8; 32];
///     type Balance = u64;
///     type Hash = [u8; 32];
///     type Timestamp = u64;
///     type BlockNumber = u64;
///     type ChainExtension = ();
/// }
///
/// type AliasWithDefaultEnv = contract_ref!(Erc20, DefaultEnvironment);
/// type AliasWithCustomEnv = contract_ref!(Erc20, CustomEnv);
/// type AliasWithGenericEnv<E> = contract_ref!(Erc20, E);
///
/// fn default(mut contract: contract_ref!(Erc20, DefaultEnvironment)) {
///     let total_supply = contract.total_supply();
///     let to: AccountId = contract.as_ref().clone();
///     contract.transfer(total_supply, to);
/// }
///
/// fn default_alias(mut contract: AliasWithDefaultEnv) {
///     default(contract)
/// }
///
/// fn custom(mut contract: contract_ref!(Erc20, CustomEnv)) {
///     let total_supply = contract.total_supply();
///     let to: [u8; 32] = contract.as_ref().clone();
///     contract.transfer(total_supply, to.into());
/// }
///
/// fn custom_alias(mut contract: AliasWithCustomEnv) {
///     custom(contract)
/// }
///
/// fn generic<E, A>(mut contract: contract_ref!(Erc20, E))
/// where
///     E: ink_env::Environment<AccountId = A>,
///     A: Into<AccountId> + Clone,
/// {
///     let total_supply = contract.total_supply();
///     let to = contract.as_ref().clone();
///     contract.transfer(total_supply, to.into());
/// }
///
/// fn generic_alias<E, A>(mut contract: AliasWithGenericEnv<E>)
/// where
///     E: ink_env::Environment<AccountId = A>,
///     A: Into<AccountId> + Clone,
/// {
///     generic(contract)
/// }
///
/// type Environment = DefaultEnvironment;
///
/// fn contract_ref_default_behaviour(mut contract: contract_ref!(Erc20)) {
///     let total_supply = contract.total_supply();
///     let to: AccountId = contract.as_ref().clone();
///     contract.transfer(total_supply, to);
/// }
/// ```
#[macro_export]
macro_rules! contract_ref {
    // The case of the default `Environment`
    ( $trait_path:path ) => {
        $crate::contract_ref!($trait_path, Environment)
    };
    // The case of the custom `Environment`
    ( $trait_path:path, $env:ty ) => {
        <<$crate::reflect::TraitDefinitionRegistry<$env> as $trait_path>
                    ::__ink_TraitInfo as $crate::codegen::TraitCallForwarder>::Forwarder
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
