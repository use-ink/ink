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

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]

extern crate proc_macro;

mod blake2b;
mod chain_extension;
mod contract;
mod event;
mod ink_test;
mod scale;
mod selector;
mod storage;
mod storage_item;
mod trait_def;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

/// Computes and expands into the BLAKE2b 256-bit hash of the string input.
///
/// # Note
///
/// - The computation takes place at compilation time of the crate.
/// - The returned value is of type `[u8; 32]`.
///
/// # Example
///
/// ```
/// # use ink_macro::blake2x256;
/// # use ink_ir::blake2b_256;
/// assert_eq!(blake2x256!("hello"), {
///     let mut output = [0u8; 32];
///     blake2b_256(b"hello", &mut output);
///     output
/// });
/// ```
#[proc_macro]
pub fn blake2x256(input: TokenStream) -> TokenStream {
    blake2b::generate_blake2x256_hash(input.into()).into()
}

/// Computes the ink! selector of the string and expands into its `u32` representation.
///
/// # Note
///
/// The computation takes place at compilation time of the crate.
///
/// # Example
///
/// ```
/// # use ink_macro::selector_id;
/// assert_eq!(selector_id!("hello"), 843960066,);
/// ```
#[proc_macro]
pub fn selector_id(input: TokenStream) -> TokenStream {
    selector::generate_selector_id(input.into()).into()
}

/// Computes the ink! selector of the string and expands into its byte representation.
///
/// # Note
///
/// The computation takes place at compilation time of the crate.
///
/// # Example
///
/// ```
/// # use ink_macro::selector_bytes;
/// assert_eq!(selector_bytes!("hello"), [50, 77, 207, 2],);
/// ```
#[proc_macro]
pub fn selector_bytes(input: TokenStream) -> TokenStream {
    selector::generate_selector_bytes(input.into()).into()
}

/// Entry point for writing ink! smart contracts.
///
/// If you are a beginner trying to learn ink! we recommend you to check out
/// our extensive [ink! workshop](https://docs.substrate.io/tutorials/v3/ink-workshop/pt1).
///
/// # Description
///
/// The macro does analysis on the provided smart contract code and generates
/// proper code.
///
/// ink! smart contracts can compile in several different modes.
/// There are two main compilation models using either
/// - on-chain mode: `no_std` and WebAssembly as target
/// - off-chain mode: `std`
///
/// We generally use the on-chain mode for actual smart contract instantiation
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
/// - `keep_attr: String`
///
///     Tells the ink! code generator which attributes should be passed to call builders.
///     Call builders are used to doing cross-contract calls and are automatically
///     generated for contracts.
///
///     **Usage Example:**
///     ```
///     #[ink::contract(keep_attr = "foo, bar")]
///     mod my_contract {
///         # #[ink(storage)]
///         # pub struct MyStorage;
///         # impl MyStorage {
///         #     #[ink(constructor)]
///         //    #[bar]
///         #     pub fn construct() -> Self { MyStorage {} }
///         #     #[ink(message)]
///         //    #[foo]
///         #     pub fn message(&self) {}
///         # }
///         // ...
///     }
///     ```
///
///     **Allowed attributes by default:** `cfg`, `cfg_attr`, `allow`, `warn`, `deny`,
///     `forbid`, `deprecated`, `must_use`, `doc`, `rustfmt`.
///
/// - `env: impl Environment`
///
///     Tells the ink! code generator which environment to use for the ink! smart
///     contract. The environment must implement the `Environment` (defined in `ink_env`)
///     trait and provides all the necessary fundamental type definitions for `Balance`,
///     `AccountId` etc.
///
///     When using a custom `Environment` implementation for a smart contract all types
///     that it exposes to the ink! smart contract and the mirrored types used in the
///     runtime must be aligned with respect to SCALE encoding and semantics.
///
///     **Usage Example:**
///
///     Given a custom `Environment` implementation:
///     ```
///     #[derive(Clone)]
///     pub struct MyEnvironment;
///
///     impl ink_env::Environment for MyEnvironment {
///         const MAX_EVENT_TOPICS: usize = 3;
///         type AccountId = [u8; 16];
///         type Balance = u128;
///         type Hash = [u8; 32];
///         type Timestamp = u64;
///         type BlockNumber = u32;
///         type ChainExtension = ::ink::env::NoChainExtension;
///     }
///     ```
///     A user might implement their ink! smart contract using the above custom
///     `Environment` implementation as demonstrated below:
///     ```
///     #[ink::contract(env = MyEnvironment)]
///     mod my_contract {
///         # #[derive(Clone)]
///         # pub struct MyEnvironment;
///         #
///         # impl ink_env::Environment for MyEnvironment {
///         #     const MAX_EVENT_TOPICS: usize = 3;
///         #     type AccountId = [u8; 16];
///         #     type Balance = u128;
///         #     type Hash = [u8; 32];
///         #     type Timestamp = u64;
///         #     type BlockNumber = u32;
///         #     type ChainExtension = ::ink::env::NoChainExtension;
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
///     **Default value:** `DefaultEnvironment` defined in `ink_env` crate.
///
/// ## Analysis
///
/// The `#[ink::contract]` macro fully analyses its input smart contract
/// against invalid arguments and structure.
///
/// Some example rules include but are not limited to:
///
/// - There must be exactly one `#[ink(storage)]` struct.
///
///     This struct defines the layout of the storage that the ink! smart contract
///     operates on. The user is able to use a variety of built-in facilities, combine
///     them in various ways or even provide their own implementations of storage data
///     structures.
///
///     For more information visit the `ink::storage` crate documentation.
///
///     **Example:**
///
///     ```
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
///     Methods flagged with `#[ink(constructor)]` are special in that they are
///     dispatchable upon contract instantiation. A contract may define multiple such
///     constructors which allow users of the contract to instantiate a contract in
///     multiple different ways.
///
///     **Example:**
///
///     Given the `Flipper` contract definition above we add an `#[ink(constructor)]`
///     as follows:
///
///     ```
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
///     upon contract invocation. The set of ink! messages defined for an ink! smart
///     contract define its API surface with which users are allowed to interact.
///
///     An ink! smart contract can have multiple such ink! messages defined.
///
///     **Note:**
///
///     - An ink! message with a `&self` receiver may only read state whereas an ink!
///       message with a `&mut self` receiver may mutate the contract's storage.
///
///     **Example:**
///
///     Given the `Flipper` contract definition above we add some `#[ink(message)]`
///     definitions as follows:
///
///     ```
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
///     An ink! message by default will reject calls that additional fund the smart
///     contract. Authors of ink! smart contracts can make an ink! message payable by
///     adding the `payable` flag to it. An example below:
///
///     Note that ink! constructors are always implicitly payable and thus cannot be
///     flagged as such.
///
///     ```
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
///         #[ink(message, payable)] // ...or specify payable inline.
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
///     # #[ink::contract]
///     # mod flipper {
///         # #[ink(storage)]
///         # pub struct Flipper {
///         #     value: bool,
///         # }
///     impl Flipper {
///         #[ink(constructor)]
///         #[ink(selector = 0xDEADBEEF)] // Works on constructors as well.
///         pub fn new(initial_value: bool) -> Self {
///             Flipper { value: false }
///         }
///
///         /// Flips the current value.
///         #[ink(message)]
///         #[ink(selector = 0xCAFEBABE)] // You can either specify selector out-of-line.
///         pub fn flip(&mut self) {
///             self.value = !self.value;
///         }
///
///         /// Returns the current value.
///         #[ink(message, selector = 0xFEEDBEEF)] // ...or specify selector inline.
///         pub fn get(&self) -> bool {
///             self.value
///         }
///     }
///     # }
///     ```
///
/// ## Interacting with the Contract Executor
///
/// The `ink_env` crate provides facilities to interact with the contract executor that
/// connects ink! smart contracts with the outer world.
///
/// For example it is possible to query the current call's caller via:
/// ```
/// # ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
/// let caller = ink_env::caller::<ink_env::DefaultEnvironment>();
/// # let _caller = caller;
/// # Ok(())
/// # }).unwrap();
/// ```
///
/// However, ink! provides a much simpler way to interact with the contract executor
/// via its environment accessor. An example below:
///
/// ```
/// #[ink::contract]
/// mod greeter {
///     use ink_env::debug_println;
///
///     #[ink(storage)]
///     pub struct Greeter;
///
///     impl Greeter {
///         #[ink(constructor)]
///         pub fn new() -> Self {
///             let caller = Self::env().caller();
///             debug_println!("thanks for instantiation {:?}", caller);
///             Greeter {}
///         }
///
///         #[ink(message, payable)]
///         pub fn fund(&self) {
///             let caller = self.env().caller();
///             let value = self.env().transferred_value();
///             debug_println!("thanks for the funding of {:?} from {:?}", value, caller);
///         }
///     }
/// }
/// ```
///
/// ## Events
///
/// An ink! smart contract may define events that it can emit during contract execution.
/// Emitting events can be used by third party tools to query information about a
/// contract's execution and state.
///
/// The following example ink! contract shows how an event `Transferred` is defined and
/// emitted in the `#[ink(constructor)]`.
///
/// ```
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
///         // more fields...
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
///             Self {
///                 total_supply: initial_supply,
///             }
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
///         /// Flips the current value of the Flipper's boolean.
///         #[ink(message)]
///         pub fn flip(&mut self) {
///             self.value = !self.value;
///         }
///
///         /// Returns the current value of the Flipper's boolean.
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
/// # Example
///
/// # Trait definition:
///
/// ```
/// # type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;
/// # type AccountId = <ink_env::DefaultEnvironment as ink_env::Environment>::AccountId;
///
/// #[ink::trait_definition]
/// pub trait Erc20 {
///     /// Returns the total supply of the ERC-20 smart contract.
///     #[ink(message)]
///     fn total_supply(&self) -> Balance;
///
///     /// Transfers balance from the caller to the given address.
///     #[ink(message)]
///     fn transfer(&mut self, amount: Balance, to: AccountId) -> bool;
///
///     // etc.
/// }
/// ```
///
/// # Trait implementation
///
/// Given the above trait definition you can implement it as shown below:
///
/// ```
/// #[ink::contract]
/// mod base_erc20 {
/// #    // We somehow cannot put the trait in the doc-test crate root due to bugs.
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
///     pub struct BaseErc20 {
///         total_supply: Balance,
///     }
///
///     impl BaseErc20 {
///         #[ink(constructor)]
///         pub fn new(initial_supply: Balance) -> Self {
///             Self { total_supply: initial_supply }
///         }
///     }
///
///     impl Erc20 for BaseErc20 {
///         /// Returns the total supply of the ERC-20 smart contract.
///         #[ink(message)]
///         fn total_supply(&self) -> Balance {
///             self.total_supply
///         }
///
///         #[ink(message)]
///         fn transfer(&mut self, amount: Balance, to: AccountId) -> bool {
///             unimplemented!()
///         }
///     }
/// }
/// ```
///
/// ## Header Arguments
///
/// The `#[ink::trait_definition]` macro can be provided with some additional
/// comma-separated header arguments:
///
/// - `namespace: String`
///
///     The namespace configuration parameter is used to influence the generated
///     selectors of the ink! trait messages. This is useful to disambiguate
///     ink! trait definitions with equal names.
///
///     **Usage Example:**
///     ```
///     #[ink::trait_definition(namespace = "foo")]
///     pub trait TraitDefinition {
///         #[ink(message)]
///         fn message1(&self);
///
///         #[ink(message, selector = 42)]
///         fn message2(&self);
///     }
///     ```
///
///     **Default value:** Empty.
///
/// - `keep_attr: String`
///
///     Tells the ink! code generator which attributes should be passed to call builders.
///     Call builders are used to doing cross-contract calls and are automatically
///     generated for contracts.
///
///     **Usage Example:**
///     ```
///     #[ink::trait_definition(keep_attr = "foo, bar")]
///     pub trait Storage {
///         #[ink(message)]
///     //  #[foo]
///         fn message1(&self);
///
///         #[ink(message)]
///     //  #[bar]
///         fn message2(&self);
///     }
///     ```
///
///     **Allowed attributes by default:** `cfg`, `cfg_attr`, `allow`, `warn`, `deny`,
///     `forbid`, `deprecated`, `must_use`, `doc`, `rustfmt`.
#[proc_macro_attribute]
pub fn trait_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    trait_def::analyze(attr.into(), item.into()).into()
}

/// Implements the necessary traits for a `struct` to be emitted as an event from a
/// contract.
///
/// By default, a signature topic will be generated for the event. This allows consumers
/// to filter and identify events of this type. Marking an event with `anonymous`
/// means no signature topic will be generated or emitted.
/// Custom signature topic can be specified with `signature_topic = <32 byte hex string>`.
///
/// `signature_topic` and `anonymous` are conflicting arguments.
///
/// # Examples
///
/// ```
/// #[ink::event]
/// pub struct MyEvent {
///     pub field: u32,
///     #[ink(topic)]
///     pub topic: [u8; 32],
/// }
///
/// // Setting `anonymous` means no signature topic will be emitted for the event.
/// #[ink::event(anonymous)]
/// pub struct MyAnonEvent {
///     pub field: u32,
///     #[ink(topic)]
///     pub topic: [u8; 32],
/// }
/// // Setting `signature_topic = <hex_string>` specifies custom signature topic.
/// #[ink::event(
///     signature_topic = "1111111111111111111111111111111111111111111111111111111111111111"
/// )]
/// pub struct MyCustomSignatureEvent {
///     pub field: u32,
///     #[ink(topic)]
///     pub topic: [u8; 32],
/// }
/// ```
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::generate(attr.into(), item.into()).into()
}

/// Prepares the type to be fully compatible and usable with the storage.
/// It implements all necessary traits and calculates the storage key for types.
/// `Packed` types don't have a storage key, but non-packed types (like `Mapping`, `Lazy`
/// etc.) require calculating the storage key during compilation.
///
/// Consider annotating structs and enums that are intended to be a part of
/// the storage with this macro. If the type is packed then the usage of the
/// macro is optional.
///
/// If the type is non-packed it is best to rely on automatic storage key
/// calculation via `ink::storage_item`.
///
/// The usage of `KEY: StorageKey` generic allows to propagate the parent's storage key to
/// the type and offset the storage key of the type. It is helpful for non-packed types
/// that can be used several times in the contract. Each field should have a unique
/// storage key, so propagation of the parent's storage key allows one to achieve it.
///
/// The macro should be called before `derive` macros because it can change the type.
///
/// All required traits can be:
/// - Derived manually via `#[derive(...)]`.
/// - Derived automatically via deriving of `scale::Decode` and `scale::Encode`.
/// - Derived via this macro.
///
/// # Example
///
/// ## Trait implementation
///
/// ```
/// use ink_prelude::vec::Vec;
/// use ink::storage::{
///     Lazy,
///     Mapping,
/// };
/// use ink::storage::traits::{
///     StorageKey,
///     StorableHint,
/// };
/// use ink::storage::traits::Storable;
///
/// // Deriving `scale::Decode` and `scale::Encode` also derives blanket implementation of all
/// // required traits to be storable.
/// #[derive(scale::Decode, scale::Encode)]
/// #[cfg_attr(
///     feature = "std",
///     derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
/// )]
/// #[derive(Default, Debug)]
/// struct Packed {
///     s1: u128,
///     s2: Vec<u128>,
///     // Fails because `StorableHint` is only implemented for `Vec` where `T: Packed`.
///     // s3: Vec<NonPacked>,
/// }
///
/// // Example of how to define the packed type with generic.
/// #[derive(scale::Decode, scale::Encode)]
/// #[cfg_attr(
///     feature = "std",
///     derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
/// )]
/// #[derive(Default, Debug)]
/// struct PackedGeneric<T: ink::storage::traits::Packed> {
///     s1: (u128, bool),
///     s2: Vec<T>,
///     s3: String,
/// }
///
/// // Example of how to define the non-packed type.
/// #[ink::storage_item]
/// #[derive(Default, Debug)]
/// struct NonPacked {
///     s1: Mapping<u32, u128>,
///     s2: Lazy<u128>,
/// }
///
/// // Example of how to define the non-packed generic type.
/// #[ink::storage_item(derive = false)]
/// #[derive(Storable, StorableHint, StorageKey)]
/// #[cfg_attr(
///     feature = "std",
///     derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
/// )]
/// #[derive(Default, Debug)]
/// struct NonPackedGeneric<T>
/// where
///     T: Default + core::fmt::Debug,
///     T: ink::storage::traits::Packed,
/// {
///     s1: u32,
///     s2: T,
///     s3: Mapping<u128, T>,
/// }
///
/// // Example of how to define a complex packed type.
/// #[derive(scale::Decode, scale::Encode)]
/// #[cfg_attr(
///     feature = "std",
///     derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
/// )]
/// #[derive(Default, Debug)]
/// struct PackedComplex {
///     s1: u128,
///     s2: Vec<u128>,
///     s3: Vec<Packed>,
/// }
///
/// // Example of how to define a complex non-packed type.
/// #[ink::storage_item]
/// #[derive(Default, Debug)]
/// struct NonPackedComplex<KEY: StorageKey> {
///     s1: (String, u128, Packed),
///     s2: Mapping<u128, u128>,
///     s3: Lazy<u128>,
///     s4: Mapping<u128, Packed>,
///     s5: Lazy<NonPacked>,
///     s6: PackedGeneric<Packed>,
///     s7: NonPackedGeneric<Packed>,
///     // Fails because: the trait `ink::storage::traits::Packed` is not implemented for `NonPacked`
///     // s8: Mapping<u128, NonPacked>,
/// }
/// ```
///
/// ## Header Arguments
///
/// The `#[ink::storage_item]` macro can be provided with an additional comma-separated
/// header argument:
///
/// - `derive: bool`
///
///     The `derive` configuration parameter is used to enable/disable auto deriving of
///     all required storage traits.
///
///     **Usage Example:**
///     ```
///     use ink::storage::Mapping;
///     use ink::storage::traits::{
///         StorableHint,
///         StorageKey,
///         Storable,
///     };
///
///     #[ink::storage_item(derive = false)]
///     #[derive(StorableHint, Storable, StorageKey)]
///     struct NonPackedGeneric<T: ink::storage::traits::Packed> {
///         s1: u32,
///         s2: Mapping<u128, T>,
///     }
///     ```
///
///     **Default value:** true.
#[proc_macro_attribute]
pub fn storage_item(attr: TokenStream, item: TokenStream) -> TokenStream {
    storage_item::generate(attr.into(), item.into()).into()
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
///     fn test2() -> Result<(), ink_env::Error> {
///         // test code that returns a Rust Result type
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    ink_test::generate(attr.into(), item.into()).into()
}

/// Defines the interface for a chain extension.
///
/// # Structure
///
/// The interface consists of an error code that indicates lightweight errors
/// as well as the definition of some chain extension methods.
///
/// The overall structure follows that of a simple Rust trait definition.
/// The error code is defined as an associated type definition of the trait definition.
/// The methods are defined as associated trait methods without implementation.
///
/// Chain extension methods must not have a `self` receiver such as `&self` or `&mut self`
/// and must have inputs and output that implement the SCALE encoding and decoding.
/// Their return value follows specific rules that can be altered using the
/// `handle_status` attribute which is described in more detail below.
///
/// # Usage
///
/// Usually the chain extension definition using this procedural macro is provided
/// by the author of the chain extension in a separate crate.
/// ink! smart contracts using this chain extension simply depend on this crate
/// and use its associated environment definition in order to make use of
/// the methods provided by the chain extension.
///
/// # Macro Attributes
///
/// The macro supports only one required argument:
///
/// - `extension = N: u16`:
///
///     The runtime may have several chain extensions at the same time. The `extension`
///     identifier points to the corresponding chain extension in the runtime.
///     The value should be the same as during the definition of the chain extension.
///
/// # Method Attributes
///
/// There are three different attributes with which the chain extension methods
/// can be flagged:
///
/// | Attribute | Required | Default Value | Description |
/// |:----------|:--------:|:--------------|:-----------:|
/// | `ink(function = N: u16)` | Yes | - | Determines the unique function ID within the
/// chain extension. | | `ink(handle_status = flag: bool)` | Optional | `true` | Assumes
/// that the returned status code of the chain extension method always indicates success
/// and therefore always loads and decodes the output buffer of the call. |
///
/// As with all ink! attributes multiple of them can either appear in a contiguous list:
/// ```
/// # type Access = i32;
/// # #[ink::chain_extension(extension = 1)]
/// # pub trait MyChainExtension {
/// #     type ErrorCode = i32;
/// #[ink(function = 5, handle_status = false)]
/// fn key_access_for_account(key: &[u8], account: &[u8]) -> Access;
/// # }
/// ```
/// …or as multiple stand alone ink! attributes applied to the same item:
/// ```
/// # type Access = i32;
/// # #[ink::chain_extension(extension = 1)]
/// # pub trait MyChainExtension {
/// #     type ErrorCode = i32;
/// #[ink(function = 5)]
/// #[ink(handle_status = false)]
/// fn key_access_for_account(key: &[u8], account: &[u8]) -> Access;
/// # }
/// ```
///
/// ## Details: `handle_status`
///
/// Default value: `true`
///
/// By default all chain extension methods should return a `Result<T, E>` where `E:
/// From<Self::ErrorCode>`. The `Self::ErrorCode` represents the error code of the chain
/// extension. This means that a smart contract calling such a chain extension method
/// first queries the returned status code of the chain extension method and only loads
/// and decodes the output if the returned status code indicates a successful call.
/// This design was chosen as it is more efficient when no output besides the error
/// code is required for a chain extension call. When designing a chain extension try to
/// utilize the error code to return errors and only use the output buffer for information
/// that does not fit in a single `u32` value.
///
/// A chain extension method that is flagged with `handle_status = false` assumes that the
/// returned error code will always indicate success. Therefore it will always load and
/// decode the output buffer and loses the `E: From<Self::ErrorCode>` constraint for the
/// call.
///
/// Note that if a chain extension method does not return `Result<T, E>` where `E:
/// From<Self::ErrorCode>` but `handle_status = true` it will still return a value of type
/// `Result<T, Self::ErrorCode>`.
///
/// ## Usage: `handle_status`
///
/// Use both `handle_status = false` and non-`Result<T, E>` return type for the same chain
/// extension method if a call to it may never fail and never returns a `Result` type.
///
/// # Combinations
///
/// Due to the possibility to flag a chain extension method with `handle_status` and
/// return or not `Result<T, E>` there are 4 different cases with slightly varying
/// semantics:
///
/// | `handle_status` | Returns `Result<T, E>` | Effects |
/// |:---------------:|:----------------:|:--------|
/// | `true`  | `true`  | The chain extension method is required to return a value of type
/// `Result<T, E>` where `E: From<Self::ErrorCode>`. A call will always check if the
/// returned status code indicates success and only then will load and decode the value in
/// the output buffer. | | `true`  | `false` | The chain extension method may return any
/// non-`Result` type. A call will always check if the returned status code indicates
/// success and only then will load and decode the value in the output buffer. The actual
/// return type of the chain extension method is still `Result<T, Self::ErrorCode>` when
/// the chain extension method was defined to return a value of type `T`. | | `false` |
/// `true`  | The chain extension method is required to return a value of type `Result<T,
/// E>`. A call will always assume that the returned status code indicates success and
/// therefore always load and decode the output buffer directly. | | `false` | `false` |
/// The chain extension method may return any non-`Result` type. A call will always assume
/// that the returned status code indicates success and therefore always load and decode
/// the output buffer directly. |
///
/// # Error Code
///
/// Every chain extension defines exactly one `ErrorCode` using the following syntax:
///
/// ```
/// #[ink::chain_extension(extension = 0)]
/// pub trait MyChainExtension {
///     type ErrorCode = MyErrorCode;
///
///     // more definitions
/// }
/// ```
///
/// The defined `ErrorCode` must implement `FromStatusCode` which should be implemented as
/// a more or less trivial conversion from the `u32` status code to a `Result<(),
/// Self::ErrorCode>`. The `Ok(())` value indicates that the call to the chain extension
/// method was successful.
///
/// By convention an error code of `0` represents success.
/// However, chain extension authors may use whatever suits their needs.
///
/// # Example: Definition
///
/// In the below example a chain extension is defined that allows its users to read and
/// write from and to the runtime storage using access privileges:
///
/// ```
/// /// Custom chain extension to read to and write from the runtime.
/// #[ink::chain_extension(extension = 0)]
/// pub trait RuntimeReadWrite {
///     type ErrorCode = ReadWriteErrorCode;
///
///     /// Reads from runtime storage.
///     ///
///     /// # Note
///     ///
///     /// Actually returns a value of type `Result<Vec<u8>, Self::ErrorCode>`.
///     #[ink(function = 1)]
///     fn read(key: &[u8]) -> Vec<u8>;
///
///     /// Reads from runtime storage.
///     ///
///     /// Returns the number of bytes read and up to 32 bytes of the
///     /// read value. Unused bytes in the output are set to 0.
///     ///
///     /// # Errors
///     ///
///     /// If the runtime storage cell stores a value that requires more than
///     /// 32 bytes.
///     ///
///     /// # Note
///     ///
///     /// This requires `ReadWriteError` to implement `From<ReadWriteErrorCode>`
///     /// and may potentially return any `Self::ErrorCode` through its return value.
///     #[ink(function = 2)]
///     fn read_small(key: &[u8]) -> Result<(u32, [u8; 32]), ReadWriteError>;
///
///     /// Writes into runtime storage.
///     ///
///     /// # Note
///     ///
///     /// Actually returns a value of type `Result<(), Self::ErrorCode>`.
///     #[ink(function = 3)]
///     fn write(key: &[u8], value: &[u8]);
///
///     /// Returns the access allowed for the key for the caller.
///     ///
///     /// # Note
///     ///
///     /// Assumes to never fail the call and therefore always returns `Option<Access>`.
///     #[ink(function = 4, handle_status = false)]
///     fn access(key: &[u8]) -> Option<Access>;
///
///     /// Unlocks previously acquired permission to access key.
///     ///
///     /// # Errors
///     ///
///     /// If the permission was not granted.
///     ///
///     /// # Note
///     ///
///     /// Assumes the call to never fail and therefore does _NOT_ require `UnlockAccessError`
///     /// to implement `From<Self::ErrorCode>` as in the `read_small` method above.
///     #[ink(function = 5, handle_status = false)]
///     fn unlock_access(key: &[u8], access: Access) -> Result<(), UnlockAccessError>;
/// }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
/// # pub enum ReadWriteErrorCode {
/// #     InvalidKey,
/// #     CannotWriteToKey,
/// #     CannotReadFromKey,
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
/// # pub enum ReadWriteError {
/// #     ErrorCode(ReadWriteErrorCode),
/// #     BufferTooSmall { required_bytes: u32 },
/// # }
/// # impl From<ReadWriteErrorCode> for ReadWriteError {
/// #     fn from(error_code: ReadWriteErrorCode) -> Self {
/// #         Self::ErrorCode(error_code)
/// #     }
/// # }
/// # impl From<scale::Error> for ReadWriteError {
/// #     fn from(_: scale::Error) -> Self {
/// #         panic!("encountered unexpected invalid SCALE encoding")
/// #     }
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
/// # pub struct UnlockAccessError {
/// #     reason: String,
/// # }
/// # impl From<scale::Error> for UnlockAccessError {
/// #     fn from(_: scale::Error) -> Self {
/// #         panic!("encountered unexpected invalid SCALE encoding")
/// #     }
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo, Clone)]
/// # pub enum Access {
/// #     ReadWrite,
/// #     ReadOnly,
/// #     WriteOnly,
/// # }
/// # impl ink_env::chain_extension::FromStatusCode for ReadWriteErrorCode {
/// #     fn from_status_code(status_code: u32) -> Result<(), Self> {
/// #         match status_code {
/// #             0 => Ok(()),
/// #             1 => Err(Self::InvalidKey),
/// #             2 => Err(Self::CannotWriteToKey),
/// #             3 => Err(Self::CannotReadFromKey),
/// #             _ => panic!("encountered unknown status code"),
/// #         }
/// #     }
/// # }
/// ```
///
/// All the error types and other utility types used in the chain extension definition
/// above are often required to implement various traits such as SCALE's `Encode` and
/// `Decode` as well as `scale-info`'s `TypeInfo` trait.
///
/// A full example of the above chain extension definition can be seen
/// [here](https://github.com/use-ink/ink/blob/017f71d60799b764425334f86b732cc7b7065fe6/crates/lang/macro/tests/ui/chain_extension/simple.rs).
///
/// # Example: Environment
///
/// In order to allow ink! smart contracts to use the above defined chain extension it
/// needs to be integrated into an `Environment` definition as shown below:
///
/// ```
/// # type RuntimeReadWrite = i32;
/// #
/// use ink_env::{
///     DefaultEnvironment,
///     Environment,
/// };
///
/// #[derive(Clone)]
/// pub enum CustomEnvironment {}
///
/// impl Environment for CustomEnvironment {
///     const MAX_EVENT_TOPICS: usize =
///         <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;
///
///     type AccountId = <DefaultEnvironment as Environment>::AccountId;
///     type Balance = <DefaultEnvironment as Environment>::Balance;
///     type Hash = <DefaultEnvironment as Environment>::Hash;
///     type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
///     type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
///
///     type ChainExtension = RuntimeReadWrite;
/// }
/// ```
///
/// Above we defined the `CustomEnvironment` which defaults to ink!'s `DefaultEnvironment`
/// for all constants and types but the `ChainExtension` type which is assigned to our
/// newly defined chain extension.
///
/// # Example: Usage
///
/// An ink! smart contract can use the above defined chain extension through the
/// `Environment` definition defined in the last example section using the `env` macro
/// parameter as shown below.
///
/// Note that chain extension methods are accessible through `Self::extension()` or
/// `self.extension()`. For example as in `Self::extension().read(...)` or
/// `self.extension().read(...)`.
///
/// ```
/// #[ink::contract(env = CustomEnvironment)]
/// mod read_writer {
///     #[ink(storage)]
///     pub struct ReadWriter {}
///
///     impl ReadWriter {
///         #[ink(constructor)]
///         pub fn new() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn read(&self, key: Vec<u8>) -> Result<Vec<u8>, ReadWriteErrorCode> {
///             self.env()
///                 .extension()
///                 .read(&key)
///         }
///
///         #[ink(message)]
///         pub fn read_small(&self, key: Vec<u8>) -> Result<(u32, [u8; 32]), ReadWriteError> {
///             self.env()
///                 .extension()
///                 .read_small(&key)
///         }
///
///         #[ink(message)]
///         pub fn write(
///             &self,
///             key: Vec<u8>,
///             value: Vec<u8>,
///         ) -> Result<(), ReadWriteErrorCode> {
///             self.env()
///                 .extension()
///                 .write(&key, &value)
///         }
///
///         #[ink(message)]
///         pub fn access(&self, key: Vec<u8>) -> Option<Access> {
///             self.env()
///                 .extension()
///                 .access(&key)
///         }
///
///         #[ink(message)]
///         pub fn unlock_access(&self, key: Vec<u8>, access: Access) -> Result<(), UnlockAccessError> {
///             self.env()
///                 .extension()
///                 .unlock_access(&key, access)
///         }
///     }
/// # /// Custom chain extension to read to and write from the runtime.
/// # #[ink::chain_extension(extension = 13)]
/// # pub trait RuntimeReadWrite {
/// #     type ErrorCode = ReadWriteErrorCode;
/// #     #[ink(function = 1)]
/// #     fn read(key: &[u8]) -> Vec<u8>;
/// #     #[ink(function = 2)]
/// #     fn read_small(key: &[u8]) -> Result<(u32, [u8; 32]), ReadWriteError>;
/// #     #[ink(function = 3)]
/// #     fn write(key: &[u8], value: &[u8]);
/// #     #[ink(function = 4, handle_status = false)]
/// #     fn access(key: &[u8]) -> Option<Access>;
/// #     #[ink(function = 5, handle_status = false)]
/// #     fn unlock_access(key: &[u8], access: Access) -> Result<(), UnlockAccessError>;
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
/// # pub enum ReadWriteErrorCode {
/// #     InvalidKey,
/// #     CannotWriteToKey,
/// #     CannotReadFromKey,
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
/// # pub enum ReadWriteError {
/// #     ErrorCode(ReadWriteErrorCode),
/// #     BufferTooSmall { required_bytes: u32 },
/// # }
/// # impl From<ReadWriteErrorCode> for ReadWriteError {
/// #     fn from(error_code: ReadWriteErrorCode) -> Self {
/// #         Self::ErrorCode(error_code)
/// #     }
/// # }
/// # impl From<scale::Error> for ReadWriteError {
/// #     fn from(_: scale::Error) -> Self {
/// #         panic!("encountered unexpected invalid SCALE encoding")
/// #     }
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
/// # pub struct UnlockAccessError {
/// #     reason: String,
/// # }
/// # impl From<scale::Error> for UnlockAccessError {
/// #     fn from(_: scale::Error) -> Self {
/// #         panic!("encountered unexpected invalid SCALE encoding")
/// #     }
/// # }
/// # #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo, Clone)]
/// # pub enum Access {
/// #     ReadWrite,
/// #     ReadOnly,
/// #     WriteOnly,
/// # }
/// # impl ink_env::chain_extension::FromStatusCode for ReadWriteErrorCode {
/// #     fn from_status_code(status_code: u32) -> Result<(), Self> {
/// #         match status_code {
/// #             0 => Ok(()),
/// #             1 => Err(Self::InvalidKey),
/// #             2 => Err(Self::CannotWriteToKey),
/// #             3 => Err(Self::CannotReadFromKey),
/// #             _ => panic!("encountered unknown status code"),
/// #         }
/// #     }
/// # }
/// # #[derive(Clone)]
/// # pub enum CustomEnvironment {}
/// # impl ink_env::Environment for CustomEnvironment {
/// #     const MAX_EVENT_TOPICS: usize =
/// #         <ink_env::DefaultEnvironment as ink_env::Environment>::MAX_EVENT_TOPICS;
/// #
/// #     type AccountId = <ink_env::DefaultEnvironment as ink_env::Environment>::AccountId;
/// #     type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;
/// #     type Hash = <ink_env::DefaultEnvironment as ink_env::Environment>::Hash;
/// #     type BlockNumber = <ink_env::DefaultEnvironment as ink_env::Environment>::BlockNumber;
/// #     type Timestamp = <ink_env::DefaultEnvironment as ink_env::Environment>::Timestamp;
/// #
/// #     type ChainExtension = RuntimeReadWrite;
/// # }
/// }
/// ```
///
/// # Technical Limitations
///
/// - Due to technical limitations it is not possible to refer to the `ErrorCode`
///   associated type using `Self::ErrorCode` anywhere within the chain extension and its
///   defined methods. Instead chain extension authors should directly use the error code
///   type when required. This limitation might be lifted in future versions of ink!.
/// - It is not possible to declare other chain extension traits as super traits or super
///   chain extensions of another.
#[proc_macro_attribute]
pub fn chain_extension(attr: TokenStream, item: TokenStream) -> TokenStream {
    chain_extension::generate(attr.into(), item.into()).into()
}

synstructure::decl_derive!(
    [Event, attributes(ink)] =>
    /// Derives an implementation of the [`ink::Event`] trait for the given `struct`.
    ///
    /// **Note** [`ink::Event`] requires [`scale::Encode`] implementation.
    ///
    /// Usually this is used in conjunction with the [`EventMetadata`] derive.
    ///
    /// For convenience there is the [`event`] attribute macro that will expand to all the necessary
    /// derives for an event implementation, including this one.
    ///
    /// # Example
    ///
    /// ```
    /// use ink::{
    ///     Event,
    ///     env::DefaultEnvironment,
    /// };
    /// use scale::Encode;
    ///
    /// #[derive(Event, Encode)]
    /// struct MyEvent {
    ///     a: u32,
    ///     #[ink(topic)]
    ///     b: [u8; 32],
    /// }
    ///
    /// #[derive(Event, Encode)]
    /// #[ink(anonymous)] // anonymous events do not have a signature topic
    /// struct MyAnonEvent {
    ///     a: u32,
    ///     #[ink(topic)]
    ///     b: [u8; 32],
    /// }
    ///
    /// ink_env::emit_event::<DefaultEnvironment, _>(MyEvent { a: 42, b: [0x42; 32] });
    /// ink_env::emit_event::<DefaultEnvironment, _>(MyAnonEvent { a: 42, b: [0x42; 32] });
    /// ```
    ///
    /// # The Signature Topic
    ///
    /// By default, the [`ink::Event::SIGNATURE_TOPIC`] is calculated as follows:
    ///
    /// `blake2b("EventStructName(field1_type_name,field2_type_name)")`
    ///
    /// The hashing of the topic is done at codegen time in the derive macro, and as such only has
    /// access to the **names** of the field types as they appear in the code. As such, if the
    /// name of a field of a struct changes, the signature topic will change too, even if the
    /// concrete type itself has not changed. This can happen with type aliases, generics, or a
    /// change in the use of a `path::to::Type` qualification.
    ///
    /// Practically this means that two otherwise identical event definitions will have different
    /// signature topics if the name of a field type differs. For example, the following two events
    /// will have different signature topics:
    ///
    /// ```
    /// #[derive(ink::Event, scale::Encode)]
    /// pub struct MyEvent {
    ///     a: u32,
    /// }
    ///
    /// mod other_event {
    ///     type MyU32 = u32;
    ///
    ///     #[derive(ink::Event, scale::Encode)]
    ///     pub struct MyEvent {
    ///         a: MyU32,
    ///     }
    /// }
    ///
    /// assert_ne!(<MyEvent as ink::env::Event>::SIGNATURE_TOPIC, <other_event::MyEvent as ink::env::Event>::SIGNATURE_TOPIC);
    /// ```
    ///
    /// ## Custom Signature
    ///
    /// Sometimes it is useful to specify the custom signature topic.
    /// For example, when the event definition from the other contract is not accessible.
    ///
    /// The macro provides `#[ink(signature_topic = _)]` nested macro that allows to provide
    /// 32 byte hex string of the custom signature topic.
    ///
    /// Generates custom signature topic
    /// ```
    /// #[derive(ink::Event, scale::Encode)]
    /// #[ink(signature_topic = "1111111111111111111111111111111111111111111111111111111111111111")]
    /// pub struct MyCustomSignatureEvent {
    ///     pub field: u32,
    ///     pub topic: [u8; 32],
    /// }
    ///
    /// assert_eq!(Some([17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17]),
    ///     <MyCustomSignatureEvent as ink::env::Event>::SIGNATURE_TOPIC)
    ///```
    /// ## Anonymous Events
    ///
    /// If the event is annotated with `#[ink(anonymous)]` then no signature topic is generated.
    /// `#[ink(signature_topic = _)]` should not be used.
    event::event_derive
);

synstructure::decl_derive!(
    [EventMetadata] =>
    /// Derives the [`ink::EventMetadata`] trait for the given `struct`, which provides metadata
    /// about the event definition.
    ///
    /// Requires that the `struct` also implements the [`ink::Event`] trait,
    /// so this derive is usually used in combination with the [`Event`] derive.
    ///
    /// Metadata is not embedded into the contract binary, it is generated from a separate
    /// compilation of the contract with the `std` feature, therefore this derive must be
    /// conditionally compiled e.g. `#[cfg_attr(feature = "std", derive(::ink::EventMetadata))]`
    /// (see example below).
    ///
    /// For convenience there is the [`event`] attribute macro that will expand to all the necessary
    /// derives for an event implementation, including this one.
    ///
    /// # Example
    ///
    /// ```
    /// use ink::{
    ///     Event,
    ///     env::DefaultEnvironment,
    /// };
    /// use scale::Encode;
    ///
    /// #[cfg_attr(feature = "std", derive(::ink::EventMetadata))]
    /// #[derive(Event, Encode)]
    /// struct MyEvent {
    ///     a: u32,
    ///     #[ink(topic)]
    ///     b: [u8; 32],
    /// }
    ///
    /// assert_eq!(<MyEvent as ink::metadata::EventMetadata>::event_spec().args().len(), 2);
    /// ```
    ///
    /// The generated code will also register this implementation with the global static distributed
    /// slice [`ink::metadata::EVENTS`], in order that the metadata of all events used in a contract
    /// can be collected.
    event::event_metadata_derive
);

synstructure::decl_derive!(
    [Storable] =>
    /// Derives `ink::storage`'s `Storable` trait for the given `struct`, `enum` or `union`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink::storage::traits::Storable;
    ///
    /// #[derive(Storable)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 1],
    /// }
    ///
    /// let value = <NamedFields as Storable>::decode(&mut &[123, 123][..]);
    /// ```
    storage::storable_derive
);
synstructure::decl_derive!(
    [StorableHint] =>
    /// Derives `ink::storage`'s `StorableHint` trait for the given `struct` or `enum`.
    ///
    /// If the type declaration contains generic `StorageKey`,
    /// it will use it as salt to generate a combined storage key.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink::storage::traits::{
    ///     Storable,
    ///     StorableHint,
    ///     StorageKey,
    ///     AutoStorableHint,
    ///     AutoKey,
    ///     ManualKey,
    /// };
    ///
    /// #[derive(Default, StorableHint, Storable)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// let _: NamedFields = <NamedFields as StorableHint<AutoKey>>::Type::default();
    /// let _: NamedFields = <NamedFields as StorableHint<ManualKey<123>>>::Type::default();
    /// ```
    storage::storable_hint_derive
);
synstructure::decl_derive!(
    [StorageKey] =>
    /// Derives `ink::storage`'s `StorageKey` trait for the given `struct` or `enum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink::storage::traits::{
    ///     AutoStorableHint,
    ///     StorageKey,
    ///     ManualKey,
    ///     AutoKey,
    /// };
    ///
    /// #[derive(StorageKey)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// assert_eq!(<NamedFields as StorageKey>::KEY, 0);
    ///
    /// #[derive(StorageKey)]
    /// struct NamedFieldsManualKey<KEY: StorageKey> {
    ///     a: <u32 as AutoStorableHint<ManualKey<0, KEY>>>::Type,
    ///     b: <[u32; 32] as AutoStorableHint<ManualKey<1, KEY>>>::Type,
    /// }
    ///
    /// assert_eq!(<NamedFieldsManualKey<()> as StorageKey>::KEY, 0);
    /// assert_eq!(<NamedFieldsManualKey<AutoKey> as StorageKey>::KEY, 0);
    /// assert_eq!(<NamedFieldsManualKey<ManualKey<123>> as StorageKey>::KEY, 123);
    /// ```
    storage::storage_key_derive
);
synstructure::decl_derive!(
    [StorageLayout] =>
    /// Derives `ink::storage`'s `StorageLayout` trait for the given `struct` or `enum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_metadata::layout::Layout::Struct;
    /// use ink::storage::traits::StorageLayout;
    ///
    /// #[derive(StorageLayout)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// let key = 0x123;
    /// let mut value = NamedFields {
    ///     a: 123,
    ///     b: [22; 32],
    /// };
    ///
    /// if let Struct(layout) = <NamedFields as StorageLayout>::layout(&key) {
    ///     assert_eq!(*layout.fields()[0].name(), "a");
    ///     assert_eq!(*layout.fields()[1].name(), "b");
    /// }
    /// ```
    storage::storage_layout_derive
);

/// Derive the re-exported traits `ink::scale::Encode`, `ink::scale::Decode` and
/// `ink::scale_info::TypeInfo`. It enables using the built in derive macros for these
/// traits without depending directly on the `parity-scale-codec` and `scale-info` crates.
///
/// # Options
///   - `Encode`: derives `ink::scale::Encode`
///   - `Decode`: derives `ink::scale::Decode`
///   - `TypeInfo`: derives `ink::scale_info::TypeInfo`
///
/// # Examples
///
/// ```
/// #[ink::scale_derive(Encode, Decode, TypeInfo)]
/// pub enum Error {}
/// ```
/// This is a convenience macro that expands to include the additional `crate` attributes
/// required for the path of the re-exported crates.
///
/// ```
/// #[derive(::ink::scale::Encode, ::ink::scale::Decode)]
/// #[codec(crate = ::ink::scale)]
/// #[cfg_attr(
///   feature = "std",
///   derive(::scale_info::TypeInfo),
///   scale_info(crate = ::ink::scale_info)
/// )]
/// pub enum Error {}
/// ```
#[proc_macro_attribute]
pub fn scale_derive(attr: TokenStream, item: TokenStream) -> TokenStream {
    match scale::derive(attr.into(), item.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
pub use contract::generate_or_err;
