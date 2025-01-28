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

/// Stores the name of the ink! smart contract.
///
/// # Note
///
/// The name is the identifier of the `#[ink(storage)]` annotated `struct`.
///
/// # Usage
///
/// ```
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self {
///             Self {}
///         }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// # use ink::reflect::ContractName;
/// assert_eq!(<Contract as ContractName>::NAME, "Contract",);
/// ```
pub trait ContractName {
    /// The name of the ink! smart contract.
    const NAME: &'static str;
}
