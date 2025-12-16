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

use alloy_sol_types::private::next_multiple_of_32;

/// Appends the given bytes to a topic (i.e. indexed event parameter) preimage buffer,
/// where the append bytes must be padded to a non-zero multiple of 32 bytes (e.g. for a
/// `string` or `bytes` member of a collection type), and returns the number of bytes
/// written.
///
/// # Panics
///
/// Panics if the preimage buffer is not large enough.
///
/// # References
///
/// - <https://docs.soliditylang.org/en/latest/abi-spec.html#events>
/// - <https://docs.soliditylang.org/en/latest/abi-spec.html#indexed-event-encoding>
pub fn append_non_empty_member_topic_bytes(bytes: &[u8], preimage: &mut [u8]) -> usize {
    let len = bytes.len();
    preimage[..len].copy_from_slice(bytes);
    let size = non_zero_multiple_of_32(len);
    if len < size {
        preimage[len..size].fill(0)
    }
    size
}

/// Same as `alloy_sol_types::utils::next_multiple_of_32` but returns `32` when n is zero.
pub fn non_zero_multiple_of_32(n: usize) -> usize {
    if n == 0 {
        return 32;
    }
    next_multiple_of_32(n)
}
