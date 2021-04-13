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

use std::collections::HashMap;

/// Provides the storage backend.
#[derive(Default)]
pub struct Storage {
    hmap: HashMap<Vec<u8>, Vec<u8>>,
}

impl Storage {
    /// Creates a new storage instance.
    pub fn new() -> Self {
        Storage {
            hmap: HashMap::new(),
        }
    }

    /// Returns the amount of entries in the storage.
    pub fn len(&self) -> usize {
        self.hmap.len()
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.hmap.get(key)
    }

    /// Removes a key from the storage, returning the value at the key if the key
    /// was previously in storage.
    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.hmap.remove(key)
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.hmap.insert(key, value)
    }

    /// Clears the storage, removing all key-value pairs.
    pub fn clear(&mut self) {
        self.hmap.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::Storage;

    #[test]
    fn basic_operations() {
        let mut storage = Storage::new();
        let key1 = vec![42];
        let key2 = vec![43];
        let val1 = vec![44];
        let val2 = vec![45];
        let val3 = vec![46];

        assert_eq!(storage.len(), 0);
        assert_eq!(storage.get(&key1), None);
        assert_eq!(storage.insert(key1.clone(), val1.clone()), None);
        assert_eq!(storage.get(&key1), Some(&val1));
        assert_eq!(
            storage.insert(key1.clone(), val2.clone()),
            Some(val1.clone())
        );
        assert_eq!(storage.get(&key1), Some(&val2));
        assert_eq!(storage.insert(key2.clone(), val3.clone()), None);
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.remove(&key2), Some(val3));
        assert_eq!(storage.len(), 1);
        storage.clear();
        assert_eq!(storage.len(), 0);
    }
}
