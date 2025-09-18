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

use ink_primitives::Address;

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
/// The macro expects one argument ‒ the path to the trait, in order to create a wrapper
/// around it.
///
/// ```rust
/// #[ink::contract]
/// mod trait_caller {
///     use ink::{
///         U256,
///         contract_ref,
///     };
///
///     #[ink::trait_definition]
///     pub trait Erc20 {
///         /// Returns the total supply of the ERC-20 smart contract.
///         #[ink(message)]
///         fn total_supply(&self) -> U256;
///
///         /// Transfers balance from the caller to the given address.
///         #[ink(message)]
///         fn transfer(&mut self, amount: U256, to: Address) -> bool;
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
///         pub fn change_account_id_1(&mut self, new_erc20: Address) {
///             self.erc20 = new_erc20.into();
///         }
///
///         /// Example of converting `AccountId` into `contract_ref!` explicitly.
///         #[ink(message)]
///         pub fn change_account_id_2(&mut self, new_erc20: Address) {
///             let erc20: contract_ref!(Erc20) = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// Example of converting `AccountId` into an alias from `contract_ref!`.
///         #[ink(message)]
///         pub fn change_account_id_3(&mut self, new_erc20: Address) {
///             type Erc20Wrapper = contract_ref!(Erc20);
///             let erc20: Erc20Wrapper = new_erc20.into();
///             self.erc20 = erc20;
///         }
///
///         /// Example of how to do common calls via fully qualified syntax.
///         #[ink(message)]
///         pub fn total_supply_1(&self) -> U256 {
///             Erc20::total_supply(&self.erc20)
///         }
///
///         /// Example of how to do common calls without fully qualified syntax.
///         #[ink(message)]
///         pub fn total_supply_2(&self) -> U256 {
///             self.erc20.total_supply()
///         }
///
///         /// Example of how to use the call builder with `contract_ref!`.
///         #[ink(message)]
///         pub fn total_supply_3(&self) -> U256 {
///             use ink::codegen::TraitCallBuilder;
///             // Returns the `CallBuilder` that implements `Erc20` trait.
///             let erc20_builder = self.erc20.call();
///             erc20_builder
///                 .total_supply()
///                 .transferred_value(U256::from(0))
///                 .invoke()
///         }
///
///         /// Example of how to do common calls and convert
///         /// the `contract_ref!` into `AccountId`.
///         #[ink(message)]
///         pub fn transfer_to_erc20(&mut self, amount: U256) -> bool {
///             let erc20_as_account_id = self.erc20.as_ref().clone();
///             self.erc20.transfer(amount, erc20_as_account_id)
///         }
///     }
/// }
/// ```
///
/// # Usage outside the `#[ink::contract]` context
///
/// The macro expects up to three arguments:
/// - The first argument is the path to the trait, e.g. `Erc20` or `erc20::Erc20`.
/// - The second argument is the type of the [`ink_env::Environment`].
/// - The third argument is the marker type for the ABI (i.e.
///   [`ink::abi::Ink`][crate::abi::Ink] or [`ink::abi::Sol`][crate::abi::Sol]).
///
/// If the second argument is not specified, the macro uses the `Environment` type alias.
/// If the third argument is not specified, the macro uses the "default" ABI for calls
/// for the ink! project.
///
/// # Note
///
/// The "default" ABI for calls is "ink", unless the ABI is set to "sol"
/// in the ink! project's manifest file (i.e. `Cargo.toml`).
///
/// ```rust
/// use ink::contract_ref;
/// use ink_env::DefaultEnvironment;
/// use ink_primitives::Address;
///
/// #[ink::trait_definition]
/// pub trait Erc20 {
///     /// Returns the total supply of the ERC-20 smart contract.
///     #[ink(message)]
///     fn total_supply(&self) -> u128;
///
///     /// Transfers balance from the caller to the given address.
///     #[ink(message)]
///     fn transfer(&mut self, amount: u128, to: Address) -> bool;
/// }
///
/// #[derive(Clone)]
/// pub struct CustomEnv;
///
/// impl ink_env::Environment for CustomEnv {
///     const NATIVE_TO_ETH_RATIO: u32 = 100_000_000;
///     type AccountId = [u8; 32];
///     type Balance = u64;
///     type Hash = [u8; 32];
///     type Timestamp = u64;
///     type BlockNumber = u64;
///     type EventRecord = ();
/// }
///
/// type AliasWithDefaultEnv = contract_ref!(Erc20, DefaultEnvironment);
/// type AliasWithCustomEnv = contract_ref!(Erc20, CustomEnv);
/// type AliasWithGenericEnv<E> = contract_ref!(Erc20, E);
/// type AliasWithCustomAbi = contract_ref!(Erc20, DefaultEnvironment, ink::abi::Ink);
///
/// fn default(mut contract: contract_ref!(Erc20, DefaultEnvironment)) {
///     let total_supply = contract.total_supply();
///     let to: Address = contract.as_ref().clone();
///     contract.transfer(total_supply, to);
/// }
///
/// fn default_alias(mut contract: AliasWithDefaultEnv) {
///     default(contract)
/// }
///
/// fn custom(mut contract: contract_ref!(Erc20, CustomEnv)) {
///     let total_supply = contract.total_supply();
///     contract.transfer(total_supply, contract.as_ref().clone());
/// }
///
/// fn custom_alias(mut contract: AliasWithCustomEnv) {
///     custom(contract)
/// }
///
/// fn generic<E, A>(mut contract: contract_ref!(Erc20, E))
/// where
///     E: ink_env::Environment<AccountId = A>,
///     A: Into<Address> + Clone,
/// {
///     let total_supply = contract.total_supply();
///     let to = contract.as_ref().clone();
///     contract.transfer(total_supply, to.into());
/// }
///
/// fn generic_alias<E, A>(mut contract: AliasWithGenericEnv<E>)
/// where
///     E: ink_env::Environment<AccountId = A>,
///     A: Into<Address> + Clone,
/// {
///     generic(contract)
/// }
///
/// fn custom_abi(mut contract: contract_ref!(Erc20, DefaultEnvironment, ink::abi::Ink)) {
///     let total_supply = contract.total_supply();
///     contract.transfer(total_supply, contract.as_ref().clone());
/// }
///
/// fn custom_alias_abi(mut contract: AliasWithCustomAbi) {
///     custom_abi(contract)
/// }
///
/// type Environment = DefaultEnvironment;
///
/// fn contract_ref_default_behaviour(mut contract: contract_ref!(Erc20)) {
///     let total_supply = contract.total_supply();
///     let to: Address = contract.as_ref().clone();
///     contract.transfer(total_supply, to);
/// }
/// ```
#[macro_export]
macro_rules! contract_ref {
    // The case of the default `Environment` and ABI
    ( $trait_path:path ) => {
        $crate::contract_ref!($trait_path, Environment)
    };
    // The case of the custom `Environment` and default ABI
    ( $trait_path:path, $env:ty ) => {
        $crate::contract_ref!($trait_path, $env, $crate::env::DefaultAbi)
    };
    // The case of the custom `Environment` and ABI
    ( $trait_path:path, $env:ty, $abi:ty ) => {
        <<$crate::reflect::TraitDefinitionRegistry<$env> as $trait_path>
                    ::__ink_TraitInfo as $crate::codegen::TraitCallForwarder>::Forwarder<$abi>
    };
}

// todo remove FromAccountId + ToAccountId
/// Implemented by contracts that are compiled as dependencies.
///
/// Allows them to return their underlying account identifier.
pub trait ToAddr {
    /// Returns the underlying account identifier of the instantiated contract.
    fn to_addr(&self) -> Address;
}
