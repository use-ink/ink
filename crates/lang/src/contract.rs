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

use crate::DispatchError;

/// The contract dispatch mode.
///
/// Tells the [`DispatchUsingMode`](`crate::DispatchUsingMode`) implementation for
/// an ink! smart contract how to dispatch for a call.
#[derive(Copy, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub enum DispatchMode {
    /// Mode for instantiating a contract.
    Instantiate,
    /// Mode for calling a contract.
    Call,
}

/// Trait implemented by contracts themselves in order to provide a clean
/// interface for the C-ABI specified `call` and `create` functions to forward
/// calls to.
#[doc(hidden)]
pub trait DispatchUsingMode {
    fn dispatch_using_mode(mode: DispatchMode) -> Result<(), DispatchError>;
}
