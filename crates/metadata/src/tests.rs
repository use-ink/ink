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

use super::*;
use pretty_assertions::assert_eq;
use scale_info::{
    IntoPortable,
    Path,
    Registry,
};
use serde_json::json;

#[test]
fn spec_constructor_selector_must_serialize_to_hex() {
    // given
    let label = "foo";
    let cs = ConstructorSpec::from_label(label)
        .selector(123_456_789u32.to_be_bytes())
        .payable(true)
        .returns(ReturnTypeSpec::new(None))
        .done();
    let mut registry = Registry::new();
    let portable_spec = cs.into_portable(&mut registry);

    // when
    let json = serde_json::to_value(&portable_spec).unwrap();
    let deserialized: ConstructorSpec<PortableForm> =
        serde_json::from_value(json.clone()).unwrap();

    // then
    assert_eq!(
        json,
        json!({
            "label": "foo",
            "payable": true,
            "selector": "0x075bcd15",
            "returnType": null,
            "args": [],
            "docs": []
        })
    );
    assert_eq!(deserialized.selector, portable_spec.selector);
}

#[test]
fn spec_contract_json() {
    // given
    let contract: ContractSpec = ContractSpec::new()
        .constructors(vec![
            ConstructorSpec::from_label("new")
                .selector([94u8, 189u8, 136u8, 214u8])
                .payable(true)
                .args(vec![MessageParamSpec::new("init_value")
                    .of_type(TypeSpec::with_name_segs::<i32, _>(
                        vec!["i32"].into_iter().map(AsRef::as_ref),
                    ))
                    .done()])
                .returns(ReturnTypeSpec::new(None))
                .docs(Vec::new())
                .done(),
            ConstructorSpec::from_label("default")
                .selector([2u8, 34u8, 255u8, 24u8])
                .payable(Default::default())
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(None))
                .docs(Vec::new())
                .done(),
            ConstructorSpec::from_label("result_new")
                .selector([6u8, 3u8, 55u8, 123u8])
                .payable(Default::default())
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(Some(TypeSpec::with_name_str::<
                    Result<(), ()>,
                >(
                    "core::result::Result"
                ))))
                .docs(Vec::new())
                .done(),
        ])
        .messages(vec![
            MessageSpec::from_label("inc")
                .selector([231u8, 208u8, 89u8, 15u8])
                .mutates(true)
                .payable(true)
                .args(vec![MessageParamSpec::new("by")
                    .of_type(TypeSpec::with_name_segs::<i32, _>(
                        vec!["i32"].into_iter().map(AsRef::as_ref),
                    ))
                    .done()])
                .returns(ReturnTypeSpec::new(None))
                .done(),
            MessageSpec::from_label("get")
                .selector([37u8, 68u8, 74u8, 254u8])
                .mutates(false)
                .payable(false)
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_segs::<i32, _>(
                    vec!["i32"].into_iter().map(AsRef::as_ref),
                )))
                .done(),
        ])
        .events(Vec::new())
        .lang_error(TypeSpec::with_name_segs::<ink_primitives::LangError, _>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(["ink", "LangError"]),
                ::core::convert::AsRef::as_ref,
            ),
        ))
        .done();

    let mut registry = Registry::new();

    // when
    let json = serde_json::to_value(contract.into_portable(&mut registry)).unwrap();

    // then
    assert_eq!(
        json,
        json!({
            "constructors": [
                {
                    "args": [
                        {
                            "label": "init_value",
                            "type": {
                                "displayName": [
                                    "i32"
                                ],
                                "type": 0
                            }
                        }
                    ],
                    "docs": [],
                    "label": "new",
                    "payable": true,
                    "returnType": null,
                    "selector": "0x5ebd88d6"
                },
                {
                    "args": [],
                    "docs": [],
                    "label": "default",
                    "payable": false,
                    "returnType": null,
                    "selector": "0x0222ff18"
                },
                {
                    "args": [],
                    "docs": [],
                    "label": "result_new",
                    "payable": false,
                    "returnType": {
                        "displayName": [
                            "core",
                            "result",
                            "Result"
                        ],
                        "type": 1
                    },
                    "selector": "0x0603377b"
                }
            ],
            "docs": [],
            "events": [],
            "lang_error": {
              "displayName": [
                "ink",
                "LangError"
              ],
              "type": 3
            },
            "messages": [
                {
                    "args": [
                        {
                            "label": "by",
                            "type": {
                                "displayName": [
                                    "i32"
                                ],
                                "type": 0
                            }
                        }
                    ],
                    "docs": [],
                    "mutates": true,
                    "payable": true,
                    "label": "inc",
                    "returnType": null,
                    "selector": "0xe7d0590f"
                },
                {
                    "args": [],
                    "docs": [],
                    "mutates": false,
                    "payable": false,
                    "label": "get",
                    "returnType": {
                        "displayName": [
                            "i32"
                        ],
                        "type": 0
                    },
                    "selector": "0x25444afe"
                }
            ],
        })
    )
}

/// Tests correct trimming of a simple comment with extra spaces
#[test]
fn trim_docs() {
    // given
    let label = "foo";
    let cs = ConstructorSpec::from_label(label)
        .selector(123_456_789u32.to_be_bytes())
        .docs(vec![" foobar      "])
        .payable(Default::default())
        .returns(ReturnTypeSpec::new(None))
        .done();
    let mut registry = Registry::new();
    let compact_spec = cs.into_portable(&mut registry);

    // when
    let json = serde_json::to_value(&compact_spec).unwrap();
    let deserialized: ConstructorSpec<PortableForm> =
        serde_json::from_value(json.clone()).unwrap();

    // then
    assert_eq!(
        json,
        json!({
            "label": "foo",
            "payable": false,
            "returnType": null,
            "selector": "0x075bcd15",
            "args": [],
            "docs": ["foobar"]
        })
    );
    assert_eq!(deserialized.docs, compact_spec.docs);
}

/// Tests correct trimming of a complex comment with a code snippet
#[test]
fn trim_docs_with_code() {
    // given
    let label = "foo";
    let cs = ConstructorSpec::from_label(label)
        .selector(123_456_789u32.to_be_bytes())
        .docs(vec![
            " Example      ",
            " ```",
            " fn test() {",
            "     \"Hello, World\"",
            " }",
            " ```",
        ])
        .payable(Default::default())
        .returns(ReturnTypeSpec::new(None))
        .done();
    let mut registry = Registry::new();
    let compact_spec = cs.into_portable(&mut registry);

    // when
    let json = serde_json::to_value(&compact_spec).unwrap();
    let deserialized: ConstructorSpec<PortableForm> =
        serde_json::from_value(json.clone()).unwrap();

    // then
    assert_eq!(
        json,
        json!({
            "label": "foo",
            "payable": false,
            "returnType": null,
            "selector": "0x075bcd15",
            "args": [],
            "docs": [
                "Example",
                "```",
                "fn test() {",
                "    \"Hello, World\"",
                "}",
                "```"
            ]
        })
    );
    assert_eq!(deserialized.docs, compact_spec.docs);
}

/// Helper for creating a constructor spec at runtime
fn runtime_constructor_spec() -> ConstructorSpec<PortableForm> {
    let path: Path<PortableForm> = Path::from_segments_unchecked(["FooType".to_string()]);
    let spec = TypeSpec::new(123.into(), path);
    let ret_spec = ReturnTypeSpec::new(None);
    let args = [MessageParamSpec::new("foo_arg".to_string())
        .of_type(spec)
        .done()];
    ConstructorSpec::from_label("foo".to_string())
        .selector(Default::default())
        .payable(true)
        .args(args)
        .docs(vec!["foo", "bar"])
        .returns(ret_spec)
        .done()
}

/// Helper for creating a message spec at runtime
fn runtime_message_spec() -> MessageSpec<PortableForm> {
    let path: Path<PortableForm> = Path::from_segments_unchecked(["FooType".to_string()]);
    let args = [MessageParamSpec::new("foo_arg".to_string())
        .of_type(TypeSpec::new(123.into(), path.clone()))
        .done()];
    let ret_spec = ReturnTypeSpec::new(TypeSpec::new(123.into(), path));
    MessageSpec::from_label("foo".to_string())
        .selector(Default::default())
        .mutates(false)
        .payable(true)
        .args(args)
        .returns(ret_spec)
        .docs(["foo".to_string(), "bar".to_string()])
        .done()
}

/// Helper for creating an event spec at runtime
fn runtime_event_spec() -> EventSpec<PortableForm> {
    let path: Path<PortableForm> =
        Path::from_segments_unchecked(["FooBarEvent".to_string()]);
    let spec = TypeSpec::new(789.into(), path);
    let args = [EventParamSpec::new("something".into())
        .of_type(spec)
        .indexed(true)
        .docs(vec![])
        .done()];
    EventSpec::new("foobar".into())
        .args(args)
        .docs(["foobar event".into()])
        .done()
}

/// Ensures constructing a `PortableForm` contract spec works at runtime
#[test]
fn construct_runtime_contract_spec() {
    let spec = ContractSpec::new()
        .constructors([runtime_constructor_spec()])
        .messages([runtime_message_spec()])
        .events([runtime_event_spec()])
        .docs(["foo".into()])
        .done();

    let constructor_spec = serde_json::to_value(&spec.constructors()[0]).unwrap();
    let expected_constructor_spec = serde_json::json!(
        {
            "label": "foo",
            "selector": "0x00000000",
            "payable": true,
            "returnType": null,
            "args": [
                {
                    "label": "foo_arg",
                    "type": {
                        "type": 123,
                        "displayName": [
                            "FooType"
                        ]
                    }
                }
            ],
            "docs": [
                "foo",
                "bar"
            ]
        }
    );
    assert_eq!(constructor_spec, expected_constructor_spec);

    let message_spec = serde_json::to_value(&spec.messages()[0]).unwrap();
    let expected_message_spec = serde_json::json!(
        {
            "label": "foo",
            "selector": "0x00000000",
            "mutates": false,
            "payable": true,
            "args": [
                {
                    "label": "foo_arg",
                    "type": {
                        "type": 123,
                        "displayName": [
                            "FooType"
                        ]
                    }
                }
            ],
            "returnType": {
                "type": 123,
                "displayName": [
                    "FooType"
                ]
            },
            "docs": [
                "foo",
                "bar"
            ]
        }
    );
    assert_eq!(message_spec, expected_message_spec);

    let event_spec = serde_json::to_value(&spec.events()[0]).unwrap();
    let expected_event_spec = serde_json::json!(
        {
            "label": "foobar",
            "args": [
                {
                    "label": "something",
                    "indexed": true,
                    "type": {
                        "type": 789,
                        "displayName": [
                            "FooBarEvent"
                        ]
                    },
                    "docs": []
                }
            ],
            "docs": [
                "foobar event"
            ]
        }
    );
    assert_eq!(event_spec, expected_event_spec);
}
