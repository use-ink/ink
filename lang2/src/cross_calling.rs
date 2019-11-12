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

use ink_core::env2::EnvTypes;

/// Implemented by contracts that are compiled as dependencies.
///
/// This allows to forward `&self` calls to a call forwarder
/// that encodes and dispatches the calls to the chain.
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
pub trait ForwardCallMut {
    /// The call forwarder that handles `&mut self` messages.
    type Forwarder;

    /// Instantiates a call forwarder to forward `&mut self` messages.
    fn call_mut(self) -> Self::Forwarder;
}

/// Implemented by contracts that are compiled as dependencies.
///
/// Allows them to return their underlying account identifier.
pub trait ToAccountId<Env>
where
    Env: EnvTypes,
{
    /// Returns the underlying account identifier of the instantiated contract.
    fn to_account_id(&self) -> <Env as EnvTypes>::AccountId;
}
