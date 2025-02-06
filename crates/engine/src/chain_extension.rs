// Copyright (C) Use Ink (UK) Ltd.
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

use super::Error;
use derive_more::From;
use std::collections::{
    hash_map::Entry,
    HashMap,
};

/// Chain extension registry.
///
/// Allows to register chain extension methods and call them.
pub struct ChainExtensionHandler {
    /// The currently registered runtime call handler.
    registered: HashMap<ExtensionId, Box<dyn ChainExtension>>,
    /// The output buffer used and reused for chain extension method call results.
    output: Vec<u8>,
}

/// The unique ID of the registered chain extension.
#[derive(
    Debug, From, scale::Encode, scale::Decode, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct ExtensionId(u16);

/// Types implementing this trait can be used as chain extensions.
///
/// This trait is only useful for testing contract via the off-chain environment.
pub trait ChainExtension {
    /// The static ID of the chain extension.
    ///
    /// # Note
    ///
    /// This is expected to return a constant value.
    fn ext_id(&self) -> u16;

    /// Calls the chain extension with the given input.
    ///
    /// Returns an error code and may fill the `output` buffer with a SCALE encoded
    /// result.
    #[allow(clippy::ptr_arg)]
    fn call(&mut self, func_id: u16, input: &[u8], output: &mut Vec<u8>) -> u32;
}

impl Default for ChainExtensionHandler {
    fn default() -> Self {
        ChainExtensionHandler::new()
    }
}

impl ChainExtensionHandler {
    /// Creates a new chain extension handler.
    ///
    /// Initialized with an empty set of chain extensions.
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
            output: Vec::new(),
        }
    }

    /// Resets the chain extension handler to uninitialized state.
    pub fn reset(&mut self) {
        self.registered.clear();
        self.output.clear();
    }

    /// Register a new chain extension.
    pub fn register(&mut self, extension: Box<dyn ChainExtension>) {
        let ext_id = extension.ext_id();
        self.registered.insert(ExtensionId::from(ext_id), extension);
    }

    /// Evaluates the chain extension with the given parameters.
    ///
    /// Upon success returns the values returned by the evaluated chain extension.
    pub fn eval(&mut self, id: u32, input: &[u8]) -> Result<(u32, &[u8]), Error> {
        self.output.clear();

        let func_id = (id & 0x0000FFFF) as u16;
        let ext_id = (id >> 16) as u16;

        let extension_id = ExtensionId::from(ext_id);
        match self.registered.entry(extension_id) {
            Entry::Occupied(occupied) => {
                let status_code =
                    occupied.into_mut().call(func_id, input, &mut self.output);
                Ok((status_code, &mut self.output))
            }
            Entry::Vacant(_vacant) => Err(Error::UnregisteredChainExtension),
        }
    }
}
