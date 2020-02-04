// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

/// Trait implemented by contracts to make them testable.
///
/// The testability comes from converting `#[ink(constructor)]`
/// functions from `&mut self` methods into making them actual
/// Rust constructors: e.g. with signatures like `fn new() -> Self`.
pub trait InstantiateTestable: Sized {
    /// The test wrapper for the contract.
    type Wrapped: core::ops::Deref<Target = Self> + core::ops::DerefMut<Target = Self>;

    /// Creates a testable instantiation of the contract.
    fn instantiate() -> Self::Wrapped;
}
