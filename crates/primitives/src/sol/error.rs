// Copyright (C) ink! contributors.
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

use ink_prelude::vec::Vec;

use crate::sol::Error;

/// Solidity ABI decode error data (if possible).
pub trait SolErrorDecode {
    /// Solidity ABI decode error data into this type.
    fn decode(data: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Solidity ABI encode as error data.
pub trait SolErrorEncode {
    /// Solidity ABI encode the value into Solidity error data.
    fn encode(&self) -> Vec<u8>;
}
