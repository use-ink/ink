// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use pdsl_core::env::{
    srml::Address,
    ContractEnv,
    Env,
};

/// Provides a safe interface to an environment given a contract state.
pub struct ExecutionEnv<State> {
    /// The contract state.
    pub state: State,
}

impl<State> ExecutionEnv<State> {
    pub const fn new(state: State) -> Self {
        ExecutionEnv { state }
    }
}

impl<State> ExecutionEnv<State> {
    pub fn caller(&self) -> Address {
        ContractEnv::caller()
    }

    pub fn r#return<T>(&self, val: T) -> !
    where
        T: parity_codec::Encode,
    {
        ContractEnv::return_(&val.encode()[..])
    }
}
