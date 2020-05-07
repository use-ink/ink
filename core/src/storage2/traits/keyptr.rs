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

use super::SpreadLayout;
use ink_primitives::Key;

/// A key pointer.
///
/// Mainly used by [`SpreadLayout`] trait in order to provide
/// a streamlined and efficient interface for accessing the underlying [`Key`].
pub struct KeyPtr {
    /// The underlying key.
    key: Key,
}

impl From<Key> for KeyPtr {
    fn from(key: Key) -> Self {
        Self { key }
    }
}

impl KeyPtr {
    /// Advances the key pointer by the same amount of the footprint of the
    /// generic type parameter of `T` and returns the old value.
    pub fn next_for<T>(&mut self) -> Key
    where
        T: SpreadLayout,
    {
        self.advance_by(<T as SpreadLayout>::FOOTPRINT)
    }

    /// Advances the key pointer by the given amount and returns the old value.
    pub fn advance_by(&mut self, amount: u64) -> Key {
        let copy = self.key;
        self.key += amount;
        copy
    }
}
