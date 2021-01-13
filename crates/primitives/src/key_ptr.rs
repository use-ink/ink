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

use crate::Key;

/// A key pointer.
///
/// This wraps a base key and provides an interface to mimic pointer arithmetics.
/// Mainly used to coordinate keys through static storage structures.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct KeyPtr {
    /// The underlying offset key.
    key: Key,
    /// The last shift performed.
    last_shift: u64,
}

impl From<Key> for KeyPtr {
    #[inline]
    fn from(key: Key) -> Self {
        Self { key, last_shift: 0 }
    }
}

impl KeyPtr {
    /// Advances the key pointer by the given amount and returns the old value.
    #[inline]
    pub fn advance_by(&mut self, new_shift: u64) -> &Key {
        let old_shift = core::mem::replace(&mut self.last_shift, new_shift);
        self.key += old_shift;
        &self.key
    }

    /// Returns the underlying offset key.
    pub fn key(&self) -> &Key {
        &self.key
    }
}
