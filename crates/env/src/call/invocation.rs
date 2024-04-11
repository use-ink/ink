// Copyright (C) Parity Technologies (UK) Ltd.
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

use core::marker::PhantomData;
use crate::Environment;
use super::{ExecutionInput, utils::ReturnType};

/// todo: create a new generated type a la ContractBuilder which produces an instance of this per message.
/// `ink::invoke!(Flip)::flip()` // returns Invoke instance
pub struct Invoke<Args, Output> {
    input: ExecutionInput<Args>,
    _output: ReturnType<Output>,
}

impl<Args, Output> Invoke<Args, Output>
where
    Args: scale::Encode,
    Output: scale::Decode
{
    /// todo: docs
    pub fn new(input: ExecutionInput<Args>) -> Self {
        Self {
            input,
            _output: ReturnType::default(),
        }
    }

    /// todo: docs
    pub fn invoke<I, E>(self, invoker: I) -> Result<ink_primitives::MessageResult<Output>, ()>
    where
        E: Environment,
        I: Invoker<E>,
    {
        invoker.invoke(&self.input)
    }
}

/// todo: docs
pub trait Invoker<E: Environment> {
    /// todo: docs
    fn invoke<Args, Output>(self, input: &ExecutionInput<Args>) -> Result<ink_primitives::MessageResult<Output>, ()>
        where
            Args: scale::Encode,
            Output: scale::Decode;
}