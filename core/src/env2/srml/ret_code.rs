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
