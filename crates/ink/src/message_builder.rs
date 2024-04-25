// Copyright (C) Parity Technologies (UK) Ltd.
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
/// This is similar to the call builder syntax accessible via the [`crate::contract_ref`]
/// macro, except that it is decoupled from the callee account id, as well as the
/// underlying execution environment. This allows it to be used in execution contexts
/// other than cross-contract calls.
///
/// # Usage outside of the `#[ink::contract]` context
///
/// The macro expects two arguments:
/// - The first argument is the path to the trait, e.g. `Erc20` or `erc20::Erc20`.
/// - The second argument is the type of the [`ink_env::Environment`].
///
/// If the second argument is not specified, the macro uses the
/// [`ink::env::DefaultEnvironment`].
///
/// ```rust
/// use ink::message_builder;
/// use ink_env::{
///     call::Executor,
///     DefaultEnvironment,
/// };
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
/// #[derive(Clone)]
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
/// /// To demonstrate implementing an execution environment agnostic executor
/// pub struct ExampleExecutor<E> {
///     marker: core::marker::PhantomData<E>,
/// }
///
/// impl<E> Executor<E> for ExampleExecutor<E>
/// where
///     E: ink_env::Environment,
/// {
///     type Error = ();
///     fn exec<Args, Output>(
///         self,
///         input: &ExecutionInput<Args>,
///     ) -> Result<MessageResult<Output>, Self::Error>
///     where
///         Args: Encode,
///         Output: Decode,
///     {
///         println!("Executing contract with input: {:?}", Encode::encode(input));
///         unimplemented!("Decode contract execution output")
///     }
/// }
///
/// fn default(executor: &ExampleExecutor<DefaultEnvironment>) {
///     let contract = message_builder!(Erc20, DefaultEnvironment);
///     let total_supply = contract.total_supply().exec(executor).unwrap();
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
macro_rules! message_builder {
    // The case of the default `Environment`
    ( $trait_path:path ) => {
        $crate::message_builder!($trait_path, $crate::env::DefaultEnvironment)
    };
    // The case of the custom `Environment`
    ( $trait_path:path, $env:ty ) => {
        <<<$crate::reflect::TraitDefinitionRegistry<$env>
                            as $trait_path>::__ink_TraitInfo
                            as $crate::codegen::TraitMessageBuilder>::MessageBuilder
                            as ::core::default::Default>::default()
    };
}
