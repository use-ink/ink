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

use super::OffHash;
use crate::env3::EnvTypes;
use ink_prelude::collections::BTreeMap;

/// The Wasm codes data base.
pub struct CodeDb {
    /// Mapping from (code)hash to Wasm blob.
    codes: BTreeMap<OffHash, Code>,
}

impl CodeDb {
    /// Creates a new empty Wasm codes database.
    pub fn new() -> Self {
        Self {
            codes: BTreeMap::new(),
        }
    }

    /// Puts the wasm code (as bytes) onto the chain and returns its code hash.
    pub fn put_code<T>(&mut self, code_hash: T::Hash, wasm_bytes: &[u8])
    where
        T: EnvTypes,
    {
        self.codes
            .insert(OffHash::new(&code_hash), Code::new(wasm_bytes.to_vec()));
    }

    /// Gets the wasm code blob associated with the given code hash if any.
    pub fn get_code<T>(&self, code_hash: T::Hash) -> Option<&Code>
    where
        T: EnvTypes,
    {
        self.codes.get(&OffHash::new(&code_hash))
    }

    /// Gets the wasm code blob associated with the given code hash if any.
    pub fn get_code_mut<T>(&mut self, code_hash: T::Hash) -> Option<&mut Code>
    where
        T: EnvTypes,
    {
        self.codes.get_mut(&OffHash::new(&code_hash))
    }
}

/// A Wasm blob on the chain.
pub struct Code {
    /// The bytes of the Wasm blob.
    wasm_bytes: Vec<u8>,
    /// The references to this Wasm blob to count usages.
    pub references: usize,
}

impl Code {
    /// Creates a new empty code.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Creates a new code from the given Wasm bytes.
    pub fn new(wasm_bytes: Vec<u8>) -> Self {
        Self {
            wasm_bytes,
            references: 0,
        }
    }

    /// Returns the Wasm bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }
}
