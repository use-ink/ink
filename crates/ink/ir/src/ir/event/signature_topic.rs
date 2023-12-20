// Copyright (C) Parity Technologies (UK) Ltd.
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

use crate::ir::blake2b_256;

/// The signature topic of an event variant.
///
/// Calculated with `blake2b("Event(field1_type,field2_type)")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignatureTopic {
    pub topic: Option<[u8; 32]>,
}

impl SignatureTopic {
    /// Computes the BLAKE-2 256-bit based signature topic from the given input bytes.
    pub fn compute(input: &[u8]) -> Self {
        assert!(
            input.len() >= 32,
            "Input array for signature topic is to short"
        );
        let mut output = [0; 32];
        blake2b_256(input, &mut output);
        Self {
            topic: Some(output),
        }
    }
}
