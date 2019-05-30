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

//! Definitions for environment types for contracts targeted at a
//! substrate chain with the default `node-runtime` configuration.

#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use srml_contract;
use node_runtime;

/// Contract environment types defined in substrate node-runtime
#[allow(unused)]
pub struct NodeRuntimeTypes;

impl ink_core::env::EnvTypes for NodeRuntimeTypes {
    type AccountId = srml_contract::AccountIdOf<node_runtime::Runtime>;
    type Balance = srml_contract::BalanceOf<node_runtime::Runtime>;
    type Hash = srml_contract::SeedOf<node_runtime::Runtime>;
    type Moment = srml_contract::MomentOf<node_runtime::Runtime>;
}
