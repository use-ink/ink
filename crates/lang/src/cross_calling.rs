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

use ink_env::Environment;

/// The type that can never be returned because it is not possible to craft an instance of it.
#[doc(hidden)]
pub enum NeverReturns {}

/// Implemented by contracts that are compiled as dependencies.
///
/// This allows to forward `&self` calls to a call forwarder
/// that encodes and dispatches the calls to the chain.
#[doc(hidden)]
pub trait ForwardCall {
    /// The call forwarder that handles `&self` messages.
    type Forwarder;

    /// Instantiates a call forwarder to forward `&self` messages.
    fn call(self) -> Self::Forwarder;
}

/// Implemented by contracts that are compiled as dependencies.
///
/// This allows to forward `&mut self` calls to a call forwarder
/// that encodes and dispatches the calls to the chain.
#[doc(hidden)]
pub trait ForwardCallMut {
    /// The call forwarder that handles `&mut self` messages.
    type Forwarder;

    /// Instantiates a call forwarder to forward `&mut self` messages.
    fn call_mut(self) -> Self::Forwarder;
}

/// Implemented by contracts that are compiled as dependencies.
///
/// Allows them to return their underlying account identifier.
pub trait ToAccountId<T>
where
    T: Environment,
{
    /// Returns the underlying account identifier of the instantiated contract.
    fn to_account_id(&self) -> <T as Environment>::AccountId;
}
