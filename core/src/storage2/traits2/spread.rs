// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use super::KeyPtr;

/// Types that can be stored to and loaded from the contract storage.
pub trait SpreadLayout {
    /// The footprint of the type.
    ///
    /// This is the number of adjunctive cells the type requires in order to
    /// be stored in the contract storage with spread layout.
    ///
    /// # Examples
    ///
    /// An instance of type `i32` requires one storage cell so its footprint is
    /// 1. An instance of type `(i32, i32)` requires 2 storage cells since a
    /// tuple or any other combined data structure always associates disjunct
    /// cells for its sub types. The same applies to arrays, e.g. `[i32; 5]`
    /// has a footprint of 5.
    const FOOTPRINT: u64;

    /// Pulls an instance of `Self` from the contract storage.
    ///
    /// The key pointer denotes the position where the instance is being pulled
    /// from within the contract storage
    ///
    /// # Note
    ///
    /// This method of pulling is depth-first: Sub-types are pulled first and
    /// construct the super-type through this procedure.
    fn pull_spread(ptr: &mut KeyPtr) -> Self;

    /// Pushes an instance of `Self` to the contract storage.
    ///
    /// - Tries to spread `Self` to as many storage cells as possible.
    /// - The key pointer denotes the position where the instance is being pushed
    /// to the contract storage.
    ///
    /// # Note
    ///
    /// This method of pushing is depth-first: Sub-types are pushed before
    /// their parent or super type.
    fn push_spread(&self, ptr: &mut KeyPtr);

    /// Clears an instance of `Self` from the contract storage.
    ///
    /// - Tries to clean `Self` from contract storage as if `self` was stored
    ///   in it using spread layout.
    /// - The key pointer denotes the position where the instance is being pushed
    /// to the contract storage.
    ///
    /// # Note
    ///
    /// This method of pushing is depth-first: Sub-types are cleared before
    /// their parent or super type.
    fn clear_spread(&self, ptr: &mut KeyPtr);
}
