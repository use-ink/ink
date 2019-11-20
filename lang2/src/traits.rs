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

use ink_core::{
    env2::{
        call::Selector,
        EnvAccess,
    },
    storage::{
        alloc::{
            AllocateUsing,
            Initialize,
        },
        Flush,
    },
};

/// Dispatchable functions that have inputs.
pub trait FnInput {
    /// The tuple-type of all inputs.
    type Input: scale::Decode + 'static;
}

/// Dispatchable functions that have an output.
pub trait FnOutput {
    /// The output type.
    type Output: scale::Encode + 'static;
}

/// The selector of dispatchable functions.
pub trait FnSelector {
    /// The selector.
    const SELECTOR: Selector;
}

/// Types implementing this are messages that may only read from storage.
pub trait Message: FnInput + FnOutput + FnSelector {
    const IS_MUT: bool;
}

/// Allows to directly access the environment mutably.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait AccessEnv<Env> {
    /// Returns a mutable access to the environment.
    fn access_env(&mut self) -> &mut EnvAccess<Env>;
}

/// Types implementing this trait are storage structs.
pub trait Storage: AllocateUsing + Initialize + Flush {}
