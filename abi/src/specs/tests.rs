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

#[test]
fn contract_spec_json() {
    // given
    let contract: ContractSpec = ContractSpec::new("incrementer")
        .constructors(vec![
            ConstructorSpec::new("new")
                .selector([94u8, 189u8, 136u8, 214u8])
                .args(vec![MessageParamSpec::new("init_value")
                    .of_type(TypeSpec::with_name_segs::<i32, _>(
                        vec!["i32"].into_iter().map(AsRef::as_ref),
                    ))
                    .done()])
                .docs(Vec::new())
                .done(),
            ConstructorSpec::new("default")
                .selector([2u8, 34u8, 255u8, 24u8])
                .args(Vec::new())
                .docs(Vec::new())
                .done(),
        ])
        .messages(vec![
            MessageSpec::new("inc")
                .selector([231u8, 208u8, 89u8, 15u8])
                .mutates(true)
                .args(vec![MessageParamSpec::new("by")
                    .of_type(TypeSpec::with_name_segs::<i32, _>(
                        vec!["i32"].into_iter().map(AsRef::as_ref),
                    ))
                    .done()])
                .docs(Vec::new())
                .returns(ReturnTypeSpec::new(None))
                .done(),
            MessageSpec::new("get")
                .selector([37u8, 68u8, 74u8, 254u8])
                .mutates(false)
                .args(Vec::new())
                .docs(Vec::new())
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_segs::<i32, _>(
                    vec!["i32"].into_iter().map(AsRef::as_ref),
                )))
                .done(),
        ])
        .events(Vec::new())
        .docs(Vec::new())
        .done();

    let mut registry = Registry::new();

    // when
    let json = serde_json::to_value(&contract.into_compact(&mut registry)).unwrap();

//    let buf = Vec::new();
//    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
//    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
//    json.serialize(&mut ser).unwrap();
//    println!("{}", String::from_utf8(ser.into_inner()).unwrap());

    // then
    assert_json_eq!(
        json,
        json!({
            "constructors": [
                {
                    "args": [
                        {
                            "name": 3,
                            "type": {
                                "display_name": [
                                    4
                                ],
                                "ty": 1
                            }
                        }
                    ],
                    "docs": [],
                    "name": 2,
                    "selector": "[\"0x5E\",\"0xBD\",\"0x88\",\"0xD6\"]"
                },
                {
                    "args": [],
                    "docs": [],
                    "name": 5,
                    "selector": "[\"0x02\",\"0x22\",\"0xFF\",\"0x18\"]"
                }
            ],
            "docs": [],
            "events": [],
            "messages": [
                {
                    "args": [
                        {
                            "name": 7,
                            "type": {
                                "display_name": [
                                    4
                                ],
                                "ty": 1
                            }
                        }
                    ],
                    "docs": [],
                    "mutates": true,
                    "name": 6,
                    "return_type": null,
                    "selector": "[\"0xE7\",\"0xD0\",\"0x59\",\"0x0F\"]"
                },
                {
                    "args": [],
                    "docs": [],
                    "mutates": false,
                    "name": 8,
                    "return_type": {
                        "display_name": [
                            4
                        ],
                        "ty": 1
                    },
                    "selector": "[\"0x25\",\"0x44\",\"0x4A\",\"0xFE\"]"
                }
            ],
            "name": 1
        })
    )
}
