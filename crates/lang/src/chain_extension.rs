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

/// Trait implemented by chain extensions.
///
/// Allows to use the `self.env().extension().my_chain_extension(..)` syntax.
///
/// # Note
///
/// This trait is automatically implemented when using `#[ink::chain_extension]` proc. macro.
pub trait ChainExtensionInstance {
    /// The type of the chain extension instance.
    type Instance;

    /// Creates a new instance of the chain extension to use methods with method chaining syntax.
    fn instantiate() -> Self::Instance;
}
