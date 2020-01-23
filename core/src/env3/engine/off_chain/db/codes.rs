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

use super::{
    OffAccountId,
    OffBalance,
    OffBlockNumber,
    OffCall,
    OffHash,
    OffMoment,
};
use crate::env3::EnvTypes;
use ink_prelude::collections::BTreeMap;

/// The Wasm codes data base.
pub struct CodeDb {
    /// Mapping from (code)hash to Wasm blob.
    codes: BTreeMap<OffHash, WasmBlob>,
}

impl CodeDb {
    /// Creates a new empty Wasm codes database.
    pub fn new() -> Self {
        Self {
            codes: BTreeMap::new(),
        }
    }

    /// Puts the wasm code (as bytes) onto the chain and returns its code hash.
    pub fn put_code<T>(wasm_bytes: &[u8]) -> T::Hash
    where
        T: EnvTypes,
    {
        todo!()
    }
}

/// A Wasm blob on the chain.
pub struct WasmBlob {
    /// The bytes of the Wasm blob.
    wasm_bytes: Vec<u8>,
    /// The references to this Wasm blob to count usages.
    references: usize,
}

impl WasmBlob {
    /// Creates a new empty Wasm blob with no usage references.
    pub fn empty() -> Self {
        Self {
            wasm_bytes: Vec::new(),
            references: 0,
        }
    }
}
