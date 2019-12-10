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

use super::*;
use assert_json_diff::assert_json_eq;
use serde_json::json;

#[test]
fn spec_constructor_selector_must_serialize_to_hex() {
    // given
    let name = "foo";
    let cs = ConstructorSpec::new(name)
        .selector(123_456_789u32.to_be_bytes())
        .done();

    let mut registry = Registry::new();

    // when
    let json = serde_json::to_value(&cs.into_compact(&mut registry)).unwrap();

    // then
    assert_json_eq!(
        json,
        json!({
            "name": 1,
            "selector": "[\"0x07\",\"0x5B\",\"0xCD\",\"0x15\"]",
            "args": [],
            "docs": []
        })
    );
}

