// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

/// Computes the BLAKE-2b 256-bit hash for the given input and stores it in output.
pub fn blake2b_256(input: &[u8], output: &mut [u8]) {
    use ::blake2::digest::{
        Update as _,
        VariableOutput as _,
    };
    let mut blake2 = blake2::VarBlake2b::new_keyed(&[], 32);
    blake2.update(input);
    blake2.finalize_variable(|result| output.copy_from_slice(result));
}
