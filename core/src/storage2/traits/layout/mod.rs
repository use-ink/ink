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

mod impls;

#[cfg(test)]
mod tests;

use crate::storage2::traits::KeyPtr;
use ink_abi::layout2::Layout;

/// Implemented by types that have a storage layout.
pub trait StorageLayout {
    /// Returns the static storage layout of `Self`.
    ///
    /// The given key pointer is guiding the allocation of static fields onto
    /// the contract storage regions.
    fn layout(key_ptr: &mut KeyPtr) -> Layout;
}
