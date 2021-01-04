// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::SpreadLayout;
use ink_primitives::Key;

/// Types that can be stored to and loaded from a single contract storage cell.
pub trait PackedLayout: SpreadLayout + scale::Encode + scale::Decode {
    /// Indicates to `self` that is has just been pulled from the storage.
    ///
    /// # Note
    ///
    /// Most types will have to implement a simple forwarding to their fields.
    /// However, some types such as [`storage::Box`](`crate::Box`)
    /// are required to perform some special handling upon receiving this signal.
    fn pull_packed(&mut self, at: &Key);

    /// Indicates to `self` that it is about to be pushed to contract storage.
    ///
    /// # Note
    ///
    /// Most types will have to implement a simple forwarding to their fields.
    /// However, some types such as [`storage::Box`](`crate::Box`)
    /// are required to perform some special handling upon receiving this signal.
    fn push_packed(&self, at: &Key);

    /// Indicates to `self` that it is about to be cleared from contract storage.
    ///
    /// # Note
    ///
    /// Most types will have to implement a simple forwarding to their fields.
    /// However, some types such as [`storage::Box`](`crate::Box`)
    /// are required to perform some special handling upon receiving this signal.
    fn clear_packed(&self, at: &Key);
}
