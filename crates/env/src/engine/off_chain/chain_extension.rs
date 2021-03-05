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

use super::OffChainError;
use crate::Result;
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

/// The unique ID of the registered chain extension method.
#[derive(
    Debug, From, scale::Encode, scale::Decode, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct ExtensionId(u32);

/// Types implementing this trait can be used as chain extensions.
///
/// This trait is only useful for testing contract via the off-chain environment.
pub trait ChainExtension {
    /// The static function ID of the chain extension.
    ///
    /// # Note
    ///
    /// This is expected to return a constant value.
    fn func_id(&self) -> u32;

    /// Calls the chain extension with the given input.
    ///
    /// Returns an error code and may fill the `output` buffer with a SCALE encoded result.
    fn call(&mut self, input: &[u8], output: &mut Vec<u8>) -> u32;
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
        let func_id = extension.func_id();
        self.registered
            .insert(ExtensionId::from(func_id), extension);
    }

    /// Evaluates the chain extension with the given parameters.
    ///
    /// Upon success returns the values returned by the evaluated chain extension.
    pub fn eval(&mut self, func_id: u32, input: &[u8]) -> Result<(u32, &[u8])> {
        self.output.clear();
        let extension_id = ExtensionId::from(func_id);
        match self.registered.entry(extension_id) {
            Entry::Occupied(occupied) => {
                let status_code = occupied.into_mut().call(input, &mut self.output);
                Ok((status_code, &mut self.output))
            }
            Entry::Vacant(_vacant) => {
                Err(OffChainError::UnregisteredChainExtension.into())
            }
        }
    }
}
