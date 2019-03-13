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

//! Public api to interact with the special testing environment.

use super::ContractEnv;

/// Returns the total number of reads to all storage entries.
pub fn total_reads() -> u64 {
    ContractEnv::total_reads()
}

/// Returns the total number of writes to all storage entries.
pub fn total_writes() -> u64 {
    ContractEnv::total_writes()
}
