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

use keccak_const::Keccak256;

/// Compile-time Solidity selector computation.
///
/// Takes function signature as a string and returns a 4 byte representation of the
/// selector.
pub const fn selector_bytes(sig: &str) -> [u8; 4] {
    let hash = keccak_256(sig.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

/// Compile-time Keccak-256 hash computation.
pub const fn keccak_256(bytes: &[u8]) -> [u8; 32] {
    Keccak256::new().update(bytes).finalize()
}
