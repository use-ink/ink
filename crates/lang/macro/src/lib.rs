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

#[proc_macro_attribute]
pub fn contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract::generate(attr.into(), item.into()).into()
}

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
