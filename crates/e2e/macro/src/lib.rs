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

mod codegen;
mod config;
mod ir;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

/// Defines an End-to-End test.
///
/// The system requirements are:
///
/// - A Substrate node with `pallet-revive` installed on the local system. You can e.g.
///   use [`ink-node`](https://github.com/use-ink/ink-node)
///   and install it on your PATH, or provide a path to an executable using the
///   `CONTRACTS_NODE` environment variable.
///
/// Before the test function is invoked the contract will be built. Any errors that occur
/// during the contract build will prevent the test function from being invoked.
///
/// ## Header Arguments
///
/// The `#[ink_e2e::test]` macro can be provided with additional arguments.
///
/// ### Custom Environment
///
/// You can specify the usage of a custom environment:
///
/// ```ignore
/// #[ink_e2e::test(environment = crate::EnvironmentWithManyTopics)]
/// ```
///
/// Our documentation contains [an explainer of what custom environments are](https://use.ink/basics/chain-environment-types).
/// For a full example [see here](https://github.com/use-ink/ink-examples/tree/v5.x.x/custom-environment).
///
/// ### Custom Backend
///
/// You can switch the E2E test to use the [DRink!](https://use.ink/basics/contract-testing/drink)
/// testing framework with this syntax:
///
/// ```
/// type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
///
/// #[ink_e2e::test(backend(runtime_only))]
/// async fn runtime_call_works() -> E2EResult<()> {
///     // ...
/// }
/// ```
///
/// In this configuration the test will not run against a node that is running in the
/// background, but against an in-process slimmed down `pallet-revive` execution
/// environment.
///
/// Please see [the page on testing with DRink!](https://use.ink/basics/contract-testing/drink)
/// in our documentation for more details.
/// For a full example [see here](https://github.com/use-ink/ink-examples/tree/v5.x.x/e2e-runtime-only-backend).
///
/// # Example
///
/// ```
/// # use ink::env::{
/// #    Environment,
/// #    DefaultEnvironment,
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// #
/// #[ink::contract]
/// mod my_module {
///     #[ink(storage)]
///     pub struct MyContract {}
///
///     impl MyContract {
///         #[ink(constructor)]
///         pub fn new() -> Self {
///             Self {}
///         }
///
///         #[ink(message)]
///         pub fn my_message(&self) {}
///     }
/// }
///
/// type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
///
/// #[ink_e2e::test]
/// async fn e2e_test(mut client: ::ink_e2e::Client<C, E>) -> E2EResult<()> {
///     // given
///     use my_module::MyContract;
///     let mut constructor = MyContract::new();
///     let contract = client
///         .instantiate("contract_transfer", &ink_e2e::bob(), &mut constructor)
///         .submit()
///         .await
///         .expect("instantiate failed");
///     let mut call_builder = contract.call_builder::<MyContract>();
///
///     // when
///     let my_message = call_builder.my_message();
///     let call_res = client
///         .call(&ink_e2e::eve(), &my_message)
///         .submit()
///         .await
///         .expect("call failed");
///
///     // then
///     assert!(call_res.is_ok());
///
///     Ok(())
/// }
/// ```
///
/// You can also build the `Keypair` type yourself, without going through
/// the pre-defined functions (`ink_e2e::alice()`, …):
///
/// ```
/// use std::str::FromStr;
/// let suri = ::ink_e2e::subxt_signer::SecretUri::from_str("//Alice").unwrap();
/// let alice = ::ink_e2e::Keypair::from_uri(&suri).unwrap();
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate(attr.into(), item.into()).into()
}

fn generate(attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    match generate_or_err(attr, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_or_err(attr: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    let test_definition = ir::InkE2ETest::new(attr, input)?;
    let codegen = codegen::InkE2ETest::from(test_definition);
    Ok(codegen.generate_code())
}
