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

use crate::Key;

/// A key pointer.
///
/// This wraps a base key and provides an interface to mimic pointer arithmetics.
/// Mainly used to coordinate keys through static storage structures.
#[derive(Debug, Copy, Clone)]
pub struct KeyPtr {
    /// The underlying offset key.
    offset: Key,
}

impl From<Key> for KeyPtr {
    fn from(key: Key) -> Self {
        Self { offset: key }
    }
}

impl KeyPtr {
    /// Advances the key pointer by the given amount and returns the old value.
    pub fn advance_by(&mut self, amount: u64) -> Key {
        let copy = self.offset;
        self.offset += amount;
        copy
    }
}
