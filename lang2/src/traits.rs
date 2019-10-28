// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

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
