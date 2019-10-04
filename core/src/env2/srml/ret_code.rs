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

use derive_more::From;

/// A return code which is the result of an external SRML call.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From)]
pub struct RetCode {
    code: u32,
}

impl RetCode {
    /// Creates a `success` indicating return code.
    pub fn success() -> Self {
        Self { code: 0 }
    }

    /// Returns `true` if `self` is success.
    pub fn is_success(self) -> bool {
        self.code == 0
    }

    /// Returns the `u32` representation of `self`.
    pub fn to_u32(self) -> u32 {
        self.code
    }
}
