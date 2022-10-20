// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use xxhash_rust::const_xxh3::xxh3_128;

/// Generate the topic for the event signature.
///
/// xxh3_128(path) ++ xxh3_128(event_variant) todo: + fields?
pub const fn event_signature_topic(
    path: &'static str,
    event_variant: &'static str,
) -> [u8; 32] {
    let p = xxh3_128(path.as_bytes()).to_be_bytes();
    // todo: add fields to signature?
    let s = xxh3_128(event_variant.as_bytes()).to_be_bytes();
    [
        p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8], p[9], p[10], p[11], p[12],
        p[13], p[14], p[15], s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], s[8], s[9],
        s[10], s[11], s[12], s[13], s[14], s[15],
    ]
}

// pub const fn event_field_topic_prefix(
//     path: &'static str,
//     event_variant: &'static str,
// ) -> [u8; 32] {
//     let path = xxh3_128(path.as_bytes());
//     let signature = xxh3_128()
// }
