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
/// - A Substrate node with `pallet-contracts` installed on the local system.
///   You can e.g. use [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node)
///   and install it on your PATH, or provide a path to an executable using the `CONTRACTS_NODE`
///   environment variable.
///
/// Before the test function is invoked the contract will have been build. Any errors
/// that occur during the contract build will prevent the test function from being
/// invoked.
///
/// ## Header Arguments
///
/// The `#[ink::e2e_test]` macro can be provided with some additional comma-separated
/// header arguments:
///
/// # Example
///
/// ```no_compile
/// # // TODO(#xxx) Remove the `no_compile`.
/// #[cfg(test)]
/// mod tests {
///     use ::ink_e2e::*;
///     type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
///
///     #[ink::e2e_test]
///     async fn e2e_test_2(mut client: ::ink_e2e::Client<C,E>) -> E2EResult<()> {
///         // given
///         let constructor = contract_transfer::constructors::new();
///         let contract_acc_id = client.instantiate(
///             &mut ::ink_e2e::alice(),
///             constructor,
///             1337,
///             None,
///         )
///         .await
///         .expect("instantiating contract failed")
///         .account_id;
///
///         // when
///         let transfer = contract_transfer::messages::give_me(120);
///         let call_res = client.call(
///             &mut ::ink_e2e::bob(),
///             contract_acc_id.clone(),
///             transfer.into(),
///             10,
///             None,
///         )
///         .await;
///
///         // then
///         assert!(call_res.is_ok());
///         Ok(())
///     }
/// }
/// ```
///
/// You can also use build the `Signer` type yourself, without going through
/// the pre-defined functions:
///
/// ```no_compile
/// let mut bob = ::ink_e2e::PairSigner::new(
///     ::ink_e2e::AccountKeyring::Bob.pair()
/// );
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
