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

//! Public api to interact with the special testing environment.

use super::ContractEnv;
use crate::env::AccountId;

/// Returns the total number of reads to all storage entries.
pub fn total_reads() -> u64 {
    ContractEnv::total_reads()
}

/// Returns the total number of writes to all storage entries.
pub fn total_writes() -> u64 {
    ContractEnv::total_writes()
}

/// Sets the caller for the next calls to the given address.
pub fn set_caller(address: AccountId) {
    ContractEnv::set_caller(address)
}
