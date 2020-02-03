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

use super::{
    OffCall,
    OffChainError,
};
use crate::env::{
    EnvTypes,
    Result,
};

/// Runtime call handler.
///
/// More generically a mapping from bytes to bytes.
pub struct RuntimeCallHandler {
    /// The currently registered runtime call handler.
    registered: Option<Box<dyn FnMut(OffCall)>>,
}

impl RuntimeCallHandler {
    /// Creates a new runtime call handler.
    ///
    /// Initialized without any handler.
    pub fn new() -> Self {
        Self { registered: None }
    }

    /// Register a runtime call handler.
    pub fn register<T, F>(&mut self, mut f: F)
    where
        T: EnvTypes,
        F: FnMut(<T as EnvTypes>::Call) + 'static,
    {
        self.registered = Some(Box::new(move |call: OffCall| {
            f(call
                .decode::<<T as EnvTypes>::Call>()
                .expect("could not decode call"))
        }));
    }

    /// Invokes the runtime with the given parameters.
    pub fn invoke<T>(&mut self, params: &T::Call) -> Result<()>
    where
        T: EnvTypes,
    {
        match &mut self.registered {
            Some(ref mut handler) => {
                handler(OffCall::new(params));
                Ok(())
            }
            None => Err(OffChainError::UnregisteredRuntimeCallHandler.into()),
        }
    }
}
