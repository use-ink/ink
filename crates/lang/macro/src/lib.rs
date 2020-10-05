// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

mod contract;
mod ink_test;
mod trait_def;

use proc_macro::TokenStream;

/// Entry point for writing ink! smart contracts.
///
/// If you are a beginner trying to learn ink! we recommend you to check out
/// our extensive [ink! workshop](https://substrate.dev/substrate-contracts-workshop/#/).
///
/// **Note:** In all below examples we will be using `ink_lang` crate aliased as just `ink`.
///           You can do this yourself by adding the following line to your code:
///           `use ink_lang as ink;`
///
/// # Description
///
/// The macro does analysis on the provided smart contract code and generates
/// proper code.
///
/// ink! smart contracts can compile in several different modes.
/// There are two main compilation models using either
/// - on-chain mode: `no_std` + WebAssembly as target
/// - off-chain mode: `std`
///
/// We generally use the on-chain mode for actual smart contract deployment
/// whereas we use the off-chain mode for smart contract testing using the
/// off-chain environment provided by the `ink_env` crate.
///
/// # Usage
///
/// ## Header Arguments
///
/// The `#[ink::contract]` macro can be provided with some additional comma-separated
/// header arguments:
///
/// - `dynamic_storage_allocator: bool`
///
///     Tells the ink! code generator to allow usage of ink!'s built-in dynamic
///     storage allocator.
///     - `true`: Use the dynamic storage allocator provided by ink!.
///     - `false`: Do NOT use the dynamic storage allocator provided by ink!.
///
///     This feature is generally only needed for smart contracts that try to model
///     their data in a way that contains storage entites within other storage
///     entities.
///     An example for this is the following type that could potentially be used
///     within a contract's storage struct definition:
///     ```
///     // A storage vector of storage vectors.
///     # use ink_storage as storage;
///     # type _unused =
///     storage::Vec<storage::Vec<i32>>
///     # ;
///     ```
///
///     **Usage Example:**
///     ```
///     # use ink_lang as ink;
///     #[ink::contract(dynamic_storage_allocator = true)]
///     mod my_contract {
///         # #[ink(storage)]
///         # pub struct MyStorage;
///         # impl MyStorage {
///         #     #[ink(constructor)]
///         #     pub fn construct() -> Self { MyStorage {} }
///         #     #[ink(message)]
///         #     pub fn message(&self) {}
///         # }
///         // ...
///     }
///     ```
///
///     **Default value:** `false`
///
/// - `compile_as_dependency: bool`
///
///     Tells the ink! code generator to always or never
///     compile the smart contract as if it was used as a dependency of another ink!
///     smart contract.
///     Normally this flag is only really useful for ink! developers who
///     want to inspect code generation of ink! smart contracts.
///     The author is not aware of any particular practical use case for users that
///     makes use of this flag.
///
///     **Usage Example:**
///     ```
///     # use ink_lang as ink;
///     #[ink::contract(compile_as_dependency = true)]
///     mod my_contract {
///         # #[ink(storage)]
///         # pub struct MyStorage;
///         # impl MyStorage {
///         #     #[ink(constructor)]
///         #     pub fn construct() -> Self { MyStorage {} }
///         #     #[ink(message)]
///         #     pub fn message(&self) {}
///         # }
///         // ...
///     }
///     ```
///
///     **Default value:** Depends on the crate feature propagation of `Cargo.toml`.
///
/// - `env: impl EnvTypes`
///
///     Tells the ink! code generator which environment to use for the ink! smart contract.
///     The environment must implement the `EnvTypes` (defined in `ink_env`) trait and provides
///     all the necessary fundamental type definitions for `Balance`, `AccountId` etc.
///
///     **Usage Example:**
///
///     Given a custom `EnvTypes` implementation:
///     ```
///     pub struct MyEnvTypes;
///
///     impl ink_env::EnvTypes for MyEnvTypes {
///         const MAX_EVENT_TOPICS: usize = 3;
///         type AccountId = u64;
///         type Balance = u128;
///         type Hash = [u8; 32];
///         type Timestamp = u64;
///         type BlockNumber = u32;
///     }
///     ```
///     A user might implement their ink! smart contract using the above custom `EnvTypes`
///     implementation as demonstrated below:
///     ```
///     # use ink_lang as ink;
///     #[ink::contract(env_types = MyEnvTypes)]
///     mod my_contract {
///         # pub struct MyEnvTypes;
///         #
///         # impl ink_env::EnvTypes for MyEnvTypes {
///         #     const MAX_EVENT_TOPICS: usize = 3;
///         #     type AccountId = u64;
///         #     type Balance = u128;
///         #     type Hash = [u8; 32];
///         #     type Timestamp = u64;
///         #     type BlockNumber = u32;
///         # }
///         #
///         # #[ink(storage)]
///         # pub struct MyStorage;
///         # impl MyStorage {
///         #     #[ink(constructor)]
///         #     pub fn construct() -> Self { MyStorage {} }
///         #     #[ink(message)]
///         #     pub fn message(&self) {}
///         # }
///         // ...
///     }
///     ```
///
///     **Default value:** `DefaultEnvTypes` defined in `ink_env` crate.
///
/// ## Anaylsis
///
/// The `#[ink::contract]` macro fully analyses its input smart contract
/// against invalid arguments and structure.
///
/// Some example rules include but are not limited to:
///
/// - There must be exactly one `#[ink(storage)]` struct.
///
///     This struct defined the layout of the storage that the ink! smart contract operates on.
///     The user is able to use a variety of built-in facitilies, combine them in various way
///     or even provide their own implementations of storage data structures.
///
///     For more information the user shall visit the `ink_storage` crate documentation.
///
///     **Example:**
///
///     ```
///     # use ink_lang as ink;
///     #[ink::contract]
///     mod flipper {
///         #[ink(storage)]
///         pub struct Flipper {
///             value: bool,
///         }
///         # impl Flipper {
///         #     #[ink(constructor)]
///         #     pub fn construct() -> Self { Flipper { value: false } }
///         #     #[ink(message)]
///         #     pub fn message(&self) {}
///         # }
///     }
///     ```
///
/// - There must be at least one `#[ink(constructor)]` defined method.
///
///     Methods flagged with `#[ink(constructor)]` are special in that they are dispatchable
///     upon contract instantiation. A contract may define multiple such constructors which
///     allow users of the contract to instantiate a contract in multiple different ways.
///
///     **Example:**
///
///     Given the `Flipper` contract definition above we add an `#[ink(constructor)]`
///     as follows:
///
///     ```
///     # use ink_lang as ink;
///     # #[ink::contract]
///     # mod flipper {
///         # #[ink(storage)]
///         # pub struct Flipper {
///         #     value: bool,
///         # }
///     impl Flipper {
///         #[ink(constructor)]
///         pub fn new(initial_value: bool) -> Self {
///             Flipper { value: false }
///         }
///         # #[ink(message)]
///         # pub fn message(&self) {}
///     }
///     # }
///     ```
///
/// - There must be at least one `#[ink(message)]` defined method.
///
///     Methods flagged with `#[ink(message)]` are special in that they are dispatchable
///     upon contract invocation. The set of ink! messages defined for an ink! smart contract
///     define the smart contract's API surface with which users are allowed to interact.
///
///     An ink! smart contract can have multiple such ink! messages defined.
///
///     **Note:**
///
///     - An ink! message with a `&self` receiver may only read state whereas an ink! message
///       with a `&mut self` receiver may mutate the contract's storage.
///
///     **Example:**
///
///     Given the `Flipper` contract definition above we add some `#[ink(message)]` definitions
///     as follows:
///
///     ```
///     # use ink_lang as ink;
///     # #[ink::contract]
///     # mod flipper {
///         # #[ink(storage)]
///         # pub struct Flipper {
///         #     value: bool,
///         # }
///     impl Flipper {
///         # #[ink(constructor)]
///         # pub fn new(initial_value: bool) -> Self {
///         #     Flipper { value: false }
///         # }
///         /// Flips the current value.
///         #[ink(message)]
///         pub fn flip(&mut self) {
///             self.value = !self.value;
///         }
///
///         /// Returns the current value.
///         #[ink(message)]
///         pub fn get(&self) -> bool {
///             self.value
///         }
///     }
///     # }
///     ```
///
///     **Payable Messages:**
///
///     An ink! message by default will reject calls that additional fund the smart contract.
///     Authors of ink! smart contracts can make an ink! message payable by adding the `payable`
///     flag to it. An example below:
///
///     ```
///     # use ink_lang as ink;
///     # #[ink::contract]
///     # mod flipper {
///         # #[ink(storage)]
///         # pub struct Flipper {
///         #     value: bool,
///         # }
///     impl Flipper {
///         # #[ink(constructor)]
///         # pub fn new(initial_value: bool) -> Self {
///         #     Flipper { value: false }
///         # }
///         /// Flips the current value.
///         #[ink(message)]
///         #[ink(payable)] // You can either specify payable out-of-line.
///         pub fn flip(&mut self) {
///             self.value = !self.value;
///         }
///
///         /// Returns the current value.
///         #[ink(message, payable)] // ... or specify payable inline.
///         pub fn get(&self) -> bool {
///             self.value
///         }
///     }
///     # }
///     ```
///
///     **Controlling the messages selector:**
///
///     Every ink! message and ink! constructor has a unique selector with which the
///     message or constructor can be uniquely identified within the ink! smart contract.
///     These selectors are mainly used to drive the contract's dispatch upon calling it.
///
///     An ink! smart contract author can control the selector of an ink! message or ink!
///     constructor using the `selector` flag. An example is shown below:
///
///     ```
///     # use ink_lang as ink;
///     # #[ink::contract]
///     # mod flipper {
///         # #[ink(storage)]
///         # pub struct Flipper {
///         #     value: bool,
///         # }
///     impl Flipper {
///         #[ink(constructor)]
///         #[ink(selector = "0xDEADBEEF")]
///         pub fn new(initial_value: bool) -> Self {
///             Flipper { value: false }
///         }
///
///         # /// Flips the current value.
///         # #[ink(message)]
///         # #[ink(payable)] // You can either specify payable out-of-line.
///         # pub fn flip(&mut self) {
///         #     self.value = !self.value;
///         # }
///         #
///         /// Returns the current value.
///         #[ink(message, selector = "0xFEEDBEEF")] // ... or specify selector inline.
///         pub fn get(&self) -> bool {
///             self.value
///         }
///     }
///     # }
///     ```
///
/// ## Interacting with the Contract Executor
///
/// The `ink_env` crate provides facitilies to interact with the contract executor that
/// connects ink! smart contracts with the outer world.
///
/// For example it is possible to query the current call's caller via:
/// ```
/// # ink_env::test::run_test::<ink_env::DefaultEnvTypes, _>(|_| {
/// let caller = ink_env::caller::<ink_env::DefaultEnvTypes>();
/// # let _caller = caller;
/// # Ok(())
/// # }).unwrap();
/// ```
///
/// However, ink! provides a much simpler way to interact with the contract executor
/// via its environment accessor. An example below:
///
/// ```
/// # use ink_lang as ink;
/// #
/// #[ink::contract]
/// mod greeter {
///     #[ink(storage)]
///     pub struct Greeter;
///
///     impl Greeter {
///         #[ink(constructor)]
///         pub fn new() -> Self {
///             let caller = Self::env().caller();
///             let message = format!("thanks for instantiation {:?}", caller);
///             ink_env::debug_println(&message);
///             Greeter {}
///         }
///
///         #[ink(message, payable)]
///         pub fn fund(&mut self) {
///             let caller = self.env().caller();
///             let value = self.env().transferred_balance();
///             let message = format!("thanks for the funding of {:?} from {:?}", value, caller);
///             ink_env::debug_println(&message);
///         }
///     }
/// }
/// ```
///
/// ## Events
///
/// An ink! smart contract may define events that it can emit during contract execution.
/// Emitting events can be used by third party tools to query information about a contract's
/// execution and state.
///
/// The following example ink! contract shows how an event `Transferred` is defined and
/// emitted in the `#[ink(constructor)]`.
///
/// ```
/// # use ink_lang as ink;
/// #
/// #[ink::contract]
/// mod erc20 {
///     /// Defines an event that is emitted every time value is transferred.
///     #[ink(event)]
///     pub struct Transferred {
///         from: Option<AccountId>,
///         to: Option<AccountId>,
///         value: Balance,
///     }
///
///     #[ink(storage)]
///     pub struct Erc20 {
///         total_supply: Balance,
///         // more fields ...
///     }
///
///     impl Erc20 {
///         #[ink(constructor)]
///         pub fn new(initial_supply: Balance) -> Self {
///             let caller = Self::env().caller();
///             Self::env().emit_event(Transferred {
///                 from: None,
///                 to: Some(caller),
///                 value: initial_supply,
///             });
///             Self { total_supply: initial_supply }
///         }
///
///         #[ink(message)]
///         pub fn total_supply(&self) -> Balance {
///             self.total_supply
///         }
///     }
/// }
/// ```
///
/// ## Example: Flipper
///
/// The below code shows the complete implementation of the so-called Flipper
/// ink! smart contract.
/// For us it acts as the "Hello, World!" of the ink! smart contracts because
/// it is minimal while still providing some more or less useful functionality.
///
/// It controls a single `bool` value that can be either `false` or `true`
/// and allows the user to flip this value using the `Flipper::flip` message
/// or retrieve the current value using `Flipper::get`.
///
/// ```
/// use ink_lang as ink;
///
/// #[ink::contract]
/// pub mod flipper {
///     #[ink(storage)]
///     pub struct Flipper {
///         value: bool,
///     }
///
///     impl Flipper {
///         /// Creates a new flipper smart contract initialized with the given value.
///         #[ink(constructor)]
///         pub fn new(init_value: bool) -> Self {
///             Self { value: init_value }
///         }
///
///         /// Flips the current value of the Flipper's bool.
///         #[ink(message)]
///         pub fn flip(&mut self) {
///             self.value = !self.value;
///         }
///
///         /// Returns the current value of the Flipper's bool.
///         #[ink(message)]
///         pub fn get(&self) -> bool {
///             self.value
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract::generate(attr.into(), item.into()).into()
}

/// Marks trait definitions to ink! as special ink! trait definitions.
///
/// There are some restrictions that apply to ink! trait definitions that
/// this macro checks. Also ink! trait definitions are required to have specialized
/// structure so that the main [`#[ink::contract]`](`macro@crate::contract`) macro can
/// properly generate code for its implementations.
///
/// # Example: Definition
///
/// ```
/// use ink_lang as ink;
/// # type Balance = <ink_env::DefaultEnvTypes as ink_env::EnvTypes>::Balance;
///
/// #[ink::trait_definition]
/// pub trait Erc20 {
///     /// Constructors a new ERC-20 compliant smart contract using the initial supply.
///     #[ink(constructor)]
///     fn new(initial_supply: Balance) -> Self;
///
///     /// Returns the total supply of the ERC-20 smart contract.
///     #[ink(message)]
///     fn total_supply(&self) -> Balance;
///
///     // etc.
/// }
/// ```
///
/// # Example: Implementation
///
/// Given the above trait definition you can implement it as shown below:
///
/// ```
/// # use ink_lang as ink;
/// #
/// #[ink::contract]
/// mod base_erc20 {
/// #    // We somehow cannot put the trait in the doc-test crate root due to bugs.
/// #    #[ink_lang::trait_definition]
/// #    pub trait Erc20 {
/// #        /// Constructors a new ERC-20 compliant smart contract using the initial supply.
/// #        #[ink(constructor)]
/// #        fn new(initial_supply: Balance) -> Self;
/// #
/// #        /// Returns the total supply of the ERC-20 smart contract.
/// #        #[ink(message)]
/// #        fn total_supply(&self) -> Balance;
/// #    }
/// #
///     #[ink(storage)]
///     pub struct BaseErc20 {
///         total_supply: Balance,
///         // etc ..
///     }
///
///     impl Erc20 for BaseErc20 {
///         #[ink(constructor)]
///         fn new(initial_supply: Balance) -> Self {
///             Self { total_supply: initial_supply }
///         }
///
///         /// Returns the total supply of the ERC-20 smart contract.
///         #[ink(message)]
///         fn total_supply(&self) -> Balance {
///             self.total_supply
///         }
///
///         // etc ..
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn trait_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    trait_def::analyze(attr.into(), item.into()).into()
}

/// Defines a unit test that makes use of ink!'s off-chain testing capabilities.
///
/// If your unit test does not require the existence of an off-chain environment
/// it is fine to not use this macro since it bears some overhead with the test.
///
/// Note that this macro is not required to run unit tests that require ink!'s
/// off-chain testing capabilities but merely improves code readability.
///
/// ## How do you find out if your test requires the off-chain environment?
///
/// Normally if the test recursively uses or invokes some contract methods that
/// call a method defined in `self.env()` or `Self::env()`.
///
/// An examples is the following:
///
/// ```no_compile
/// let caller: AccountId = self.env().caller();
/// ```
///
/// # Example
///
/// ```
/// use ink_lang as ink;
///
/// #[cfg(test)]
/// mod tests {
///     // Conventional unit test that works with assertions.
///     #[ink::test]
///     fn test1() {
///         // test code comes here as usual
///     }
///
///     // Conventional unit test that returns some Result.
///     // The test code can make use of operator-`?`.
///     #[ink::test]
///     fn test2() -> Result<(), ink_env::EnvError> {
///         // test code that returns a Rust Result type
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    ink_test::generate(attr.into(), item.into()).into()
}

#[cfg(test)]
pub use contract::generate_or_err;
