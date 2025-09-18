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

/// Creates an instance of a message builder for an `#[ink::trait_definition]`.
///
/// This is done by creating a wrapper around the trait defined with the
/// [`ink::trait_definition`](crate::trait_definition) macro.
///
/// The macro returns an instance of the generated message builder type which implements
/// the trait, allowing the user to create and invoke messages on the trait.
///
/// This is similar to the call builder syntax accessible via the [`crate::contract_ref!`]
/// macro, except that it is decoupled from the callee account id, as well as the
/// underlying execution environment. This allows it to be used in execution contexts
/// other than cross-contract calls.
///
/// # Usage
///
/// The macro expects up to three arguments:
/// - The first argument is the path to the trait, e.g. `Erc20` or `erc20::Erc20`.
/// - The second argument is the type of the [`ink_env::Environment`].
/// - The third argument is the marker type for the ABI (i.e.
///   [`ink::abi::Ink`][crate::abi::Ink] or [`ink::abi::Sol`][crate::abi::Sol]).
///
/// If the second argument is not specified, the macro uses the
/// [`ink_env::DefaultEnvironment`].
/// If the third argument is not specified, the macro uses the "default" ABI for calls
/// for the ink! project.
///
/// # Note
///
/// The "default" ABI for calls is "ink", unless the ABI is set to "sol"
/// in the ink! project's manifest file (i.e. `Cargo.toml`).
///
/// ```rust
/// use ink::message_builder;
/// use ink_env::{
///     DefaultEnvironment,
///     call::{
///         ExecutionInput,
///         Executor,
///     },
/// };
/// use ink_primitives::{
///     AccountId,
///     Address,
///     MessageResult,
/// };
/// use scale::{
///     Decode,
///     Encode,
/// };
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
/// /// To demonstrate implementing an execution environment agnostic executor
/// pub struct ExampleExecutor<E> {
///     marker: core::marker::PhantomData<E>,
/// }
///
/// impl<E> ExampleExecutor<E> {
///     pub fn new() -> Self {
///         Self {
///             marker: core::marker::PhantomData,
///         }
///     }
/// }
///
/// impl<E> Executor<E> for ExampleExecutor<E>
/// where
///     E: ink_env::Environment,
/// {
///     type Error = ();
///     fn exec<Args, Output, Abi>(
///         &self,
///         input: &ExecutionInput<Args, Abi>,
///     ) -> Result<MessageResult<Output>, Self::Error>
///     where
///         Args: ink::abi::AbiEncodeWith<Abi>,
///         Output: ink::env::call::utils::DecodeMessageResult<Abi>,
///     {
///         println!("Executing contract with input: {:?}", input.encode());
///         unimplemented!("Decode contract execution output")
///     }
/// }
///
/// fn default(to: Address) {
///     let executor = ExampleExecutor::<DefaultEnvironment>::new();
///     let mut contract = message_builder!(Erc20);
///     let total_supply = contract.total_supply().exec(&executor).unwrap().unwrap();
///     contract.transfer(total_supply, to).exec(&executor).unwrap();
/// }
///
/// fn custom(to: Address) {
///     let executor = ExampleExecutor::<CustomEnv>::new();
///     let mut contract = message_builder!(Erc20, CustomEnv);
///     let total_supply = contract.total_supply().exec(&executor).unwrap().unwrap();
///     contract.transfer(total_supply, to).exec(&executor).unwrap();
/// }
///
/// fn custom_abi(to: Address) {
///     let executor = ExampleExecutor::<DefaultEnvironment>::new();
///     let mut contract = message_builder!(Erc20, DefaultEnvironment, ink::abi::Ink);
///     let total_supply = contract.total_supply().exec(&executor).unwrap().unwrap();
///     contract.transfer(total_supply, to).exec(&executor).unwrap();
/// }
///
/// fn generic<E>(to: Address)
/// where
///     E: ink_env::Environment,
/// {
///     let executor = ExampleExecutor::<E>::new();
///     let mut contract = message_builder!(Erc20, E);
///     let total_supply = contract.total_supply().exec(&executor).unwrap().unwrap();
///     contract.transfer(total_supply, to).exec(&executor).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! message_builder {
    // The case of the default `Environment` and ABI
    ( $trait_path:path ) => {
        $crate::message_builder!($trait_path, $crate::env::DefaultEnvironment)
    };
    // The case of the custom `Environment` and default ABI
    ( $trait_path:path, $env:ty ) => {
        $crate::message_builder!($trait_path, $env, $crate::env::DefaultAbi)
    };
    // The case of the custom `Environment` and ABI
    ( $trait_path:path, $env:ty, $abi:ty ) => {
        <<<$crate::reflect::TraitDefinitionRegistry<$env>
                            as $trait_path>::__ink_TraitInfo
                            as $crate::codegen::TraitMessageBuilder>::MessageBuilder<$abi>
                            as ::core::default::Default>::default()
    };
}
