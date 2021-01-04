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
use std::collections::HashMap;

type FuncId = u32;

/// Types implementing this trait can be used as chain extensions.
///
/// This trait is only useful for testing contract via the off-chain environment.
pub trait ChainExtension {
    /// The expected input type.
    ///
    /// # Note
    ///
    /// This can be a tuple to expect multiple input types.
    type Input: scale::Codec;
    /// The expected output type.
    type Output: scale::Codec;

    /// The static function ID of the chain extension.
    ///
    /// # Note
    ///
    /// This is expected to return a constant value.
    fn func_id(&self) -> u32;

    /// Calls the chain extension with the given input.
    fn call(&mut self, input: &Self::Input) -> Result<Self::Output>;
}

/// A raw chain extension function.
///
/// This is mostly a wrapper closure around the real chain extension function
/// that handles marshalling of types between their encoded and decoded
/// representations.
type ChainExtensionFn = Box<dyn FnMut(Vec<u8>) -> Result<Vec<u8>>>;

/// Runtime call handler.
///
/// More generically a mapping from bytes to bytes.
pub struct ChainExtensionHandler {
    /// The currently registered runtime call handler.
    registered: HashMap<FuncId, ChainExtensionFn>,
}

impl ChainExtensionHandler {
    /// Creates a new chain extension handler.
    ///
    /// Initialized with an empty set of chain extensions.
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
        }
    }

    /// Resets the chain extension handler to uninitialized state.
    pub fn reset(&mut self) {
        self.registered.clear()
    }

    /// Register a new chain extension.
    pub fn register<I, O>(
        &mut self,
        mut extension: Box<dyn ChainExtension<Input = I, Output = O>>,
    ) where
        I: scale::Codec + 'static,
        O: scale::Codec + 'static,
    {
        let func_id = extension.func_id();
        self.registered.insert(
            func_id,
            Box::new(move |encoded_input: Vec<u8>| {
                let decoded_input = scale::Decode::decode(&mut &encoded_input[..])?;
                let decoded_output = extension.call(&decoded_input)?;
                Ok(scale::Encode::encode(&decoded_output))
            }),
        );
    }

    /// Evaluates the chain extension with the given parameters.
    ///
    /// Upon success returns the values returned by the evaluated chain extension.
    pub fn eval<I, O>(&mut self, func_id: FuncId, input: &I) -> Result<O>
    where
        I: scale::Codec + 'static,
        O: scale::Codec + 'static,
    {
        use std::collections::hash_map::Entry;
        match self.registered.entry(func_id) {
            Entry::Occupied(mut occupied) => {
                let encoded_input = scale::Encode::encode(input);
                let encoded_output = occupied.get_mut()(encoded_input)?;
                scale::Decode::decode(&mut &encoded_output[..]).map_err(Into::into)
            }
            Entry::Vacant(_vacant) => {
                Err(OffChainError::UnregisteredChainExtension.into())
            }
        }
    }
}
