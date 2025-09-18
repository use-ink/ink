// Copyright (C) Use Ink (UK) Ltd.
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
    let selector_id = 123_456_789u32;
    let label = "foo";
    let cs = ConstructorSpec::from_label(label)
        .selector(selector_id.to_be_bytes())
        .payable(true)
        .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
            ink_primitives::ConstructorResult<()>,
        >(
            "ink_primitives::ConstructorResult"
        )))
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
            "returnType": {
                "displayName": [
                    "ink_primitives",
                    "ConstructorResult"
                ],
                "type": 0
            },
            "args": [],
            "docs": [],
            "default": false,
        })
    );
    assert_eq!(deserialized.selector, portable_spec.selector);
}

#[test]
#[should_panic(expected = "only one default message is allowed")]
fn spec_contract_only_one_default_message_allowed() {
    ContractSpec::new()
        .constructors(vec![
            ConstructorSpec::from_label("new")
                .selector([94u8, 189u8, 136u8, 214u8])
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("init_value")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<()>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .done(),
        ])
        .messages(vec![
            MessageSpec::from_label("inc")
                .selector([231u8, 208u8, 89u8, 15u8])
                .mutates(true)
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("by")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::MessageResult<()>,
                >(
                    "ink_primitives::MessageResult"
                )))
                .default(true)
                .done(),
            MessageSpec::from_label("get")
                .selector([37u8, 68u8, 74u8, 254u8])
                .mutates(false)
                .payable(false)
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_segs::<i32, _>(
                    vec!["i32"].into_iter().map(AsRef::as_ref),
                )))
                .default(true)
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
}

#[test]
#[should_panic(expected = "only one default constructor is allowed")]
fn spec_contract_only_one_default_constructor_allowed() {
    ContractSpec::new()
        .constructors(vec![
            ConstructorSpec::from_label("new")
                .selector([94u8, 189u8, 136u8, 214u8])
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("init_value")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<()>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .default(true)
                .done(),
            ConstructorSpec::from_label("default")
                .selector([2u8, 34u8, 255u8, 24u8])
                .payable(Default::default())
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<()>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .default(true)
                .done(),
        ])
        .messages(vec![
            MessageSpec::from_label("inc")
                .selector([231u8, 208u8, 89u8, 15u8])
                .mutates(true)
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("by")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::MessageResult<()>,
                >(
                    "ink_primitives::MessageResult"
                )))
                .default(true)
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
}

#[test]
#[should_panic(
    expected = "event signature topic collision: `path::to::Event`, `path::to::Event2`"
)]
fn spec_contract_event_definition_signature_topic_collision() {
    const SIGNATURE_TOPIC: Option<[u8; 32]> = Some([42u8; 32]);
    const BUFFER_SIZE: usize = 1 << 14;
    const NATIVE_TO_ETH_RATIO: u32 = 100000000;

    ContractSpec::new()
        .constructors(vec![
            ConstructorSpec::from_label("new")
                .selector([94u8, 189u8, 136u8, 214u8])
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("init_value")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<()>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .default(true)
                .done(),
        ])
        .messages(vec![
            MessageSpec::from_label("inc")
                .selector([231u8, 208u8, 89u8, 15u8])
                .mutates(true)
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("by")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::MessageResult<()>,
                >(
                    "ink_primitives::MessageResult"
                )))
                .default(true)
                .done(),
        ])
        .events(vec![
            EventSpec::new("Event")
                .module_path("path::to")
                // has a signature topic which counts towards the limit
                .signature_topic(SIGNATURE_TOPIC)
                .args([])
                .done(),
            EventSpec::new("Event2")
                .module_path("path::to")
                .signature_topic::<[u8; 32]>(SIGNATURE_TOPIC)
                .args([])
                .done(),
        ])
        .lang_error(TypeSpec::with_name_segs::<ink_primitives::LangError, _>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(["ink", "LangError"]),
                ::core::convert::AsRef::as_ref,
            ),
        ))
        .environment(
            EnvironmentSpec::new()
                .account_id(TypeSpec::of_type::<ink_primitives::AccountId>())
                .balance(TypeSpec::of_type::<u128>())
                .hash(TypeSpec::of_type::<ink_primitives::Hash>())
                .timestamp(TypeSpec::of_type::<u64>())
                .block_number(TypeSpec::of_type::<u128>())
                .native_to_eth_ratio(NATIVE_TO_ETH_RATIO)
                .static_buffer_size(BUFFER_SIZE)
                .done(),
        )
        .done();
}

#[test]
fn spec_contract_json() {
    type AccountId = ink_primitives::AccountId;
    type Balance = u64;
    type Hash = ink_primitives::Hash;
    type Timestamp = u64;
    type BlockNumber = u128;
    const NATIVE_TO_ETH_RATIO: u32 = 100000000;
    const BUFFER_SIZE: usize = 1 << 14;

    // given
    let contract: ContractSpec = ContractSpec::new()
        .constructors(vec![
            ConstructorSpec::from_label("new")
                .selector([94u8, 189u8, 136u8, 214u8])
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("init_value")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<()>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .done(),
            ConstructorSpec::from_label("default")
                .selector([2u8, 34u8, 255u8, 24u8])
                .payable(Default::default())
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<()>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .default(true)
                .done(),
            ConstructorSpec::from_label("result_new")
                .selector([6u8, 3u8, 55u8, 123u8])
                .payable(Default::default())
                .args(Vec::new())
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::ConstructorResult<Result<(), ()>>,
                >(
                    "ink_primitives::ConstructorResult"
                )))
                .docs(Vec::new())
                .done(),
        ])
        .messages(vec![
            MessageSpec::from_label("inc")
                .selector([231u8, 208u8, 89u8, 15u8])
                .mutates(true)
                .payable(true)
                .args(vec![
                    MessageParamSpec::new("by")
                        .of_type(TypeSpec::with_name_segs::<i32, _>(
                            vec!["i32"].into_iter().map(AsRef::as_ref),
                        ))
                        .done(),
                ])
                .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
                    ink_primitives::MessageResult<()>,
                >(
                    "ink_primitives::MessageResult"
                )))
                .default(true)
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
        .environment(
            EnvironmentSpec::new()
                .account_id(TypeSpec::with_name_segs::<AccountId, _>(
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(["AccountId"]),
                        ::core::convert::AsRef::as_ref,
                    ),
                ))
                .balance(TypeSpec::with_name_segs::<Balance, _>(
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(["Balance"]),
                        ::core::convert::AsRef::as_ref,
                    ),
                ))
                .hash(TypeSpec::with_name_segs::<Hash, _>(
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(["Hash"]),
                        ::core::convert::AsRef::as_ref,
                    ),
                ))
                .timestamp(TypeSpec::with_name_segs::<Timestamp, _>(
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(["Timestamp"]),
                        ::core::convert::AsRef::as_ref,
                    ),
                ))
                .block_number(TypeSpec::with_name_segs::<BlockNumber, _>(
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(["BlockNumber"]),
                        ::core::convert::AsRef::as_ref,
                    ),
                ))
                .native_to_eth_ratio(NATIVE_TO_ETH_RATIO)
                .static_buffer_size(BUFFER_SIZE)
                .done(),
        )
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
                    "default": false,
                    "label": "new",
                    "payable": true,
                    "returnType": {
                        "displayName": [
                            "ink_primitives",
                            "ConstructorResult"
                        ],
                        "type": 1
                    },
                    "selector": "0x5ebd88d6"
                },
                {
                    "args": [],
                    "docs": [],
                    "default": true,
                    "label": "default",
                    "payable": false,
                    "returnType": {
                        "displayName": [
                            "ink_primitives",
                            "ConstructorResult"
                        ],
                        "type": 1
                    },
                    "selector": "0x0222ff18"
                },
                {
                    "args": [],
                    "docs": [],
                    "default": false,
                    "label": "result_new",
                    "payable": false,
                    "returnType": {
                        "displayName": [
                            "ink_primitives",
                            "ConstructorResult"
                        ],
                        "type": 4
                    },
                    "selector": "0x0603377b"
                }
            ],
            "docs": [],
            "environment":  {
                "accountId":  {
                    "displayName":  [
                        "AccountId",
                    ],
                    "type": 6,
                },
                "balance":  {
                    "displayName":  [
                        "Balance",
                    ],
                    "type": 9,
                },
                "blockNumber":  {
                    "displayName":  [
                        "BlockNumber",
                    ],
                    "type": 11,
                },
                "staticBufferSize": 16384,
                "hash":  {
                    "displayName":  [
                        "Hash",
                    ],
                    "type": 10,
                },
                "nativeToEthRatio": 100000000,
                "timestamp":  {
                    "displayName":  [
                        "Timestamp",
                    ],
                    "type": 9,
                },
            },
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
                    "default": true,
                    "docs": [],
                    "mutates": true,
                    "payable": true,
                    "label": "inc",
                    "returnType": {
                        "displayName": [
                            "ink_primitives",
                            "MessageResult"
                        ],
                        "type": 1
                    },
                    "selector": "0xe7d0590f"
                },
                {
                    "args": [],
                    "default": false,
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
        .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
            ink_primitives::ConstructorResult<()>,
        >(
            "ink_primitives::ConstructorResult"
        )))
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
            "returnType": {
                "displayName": [
                    "ink_primitives",
                    "ConstructorResult"
                ],
                "type": 0
            },
            "selector": "0x075bcd15",
            "args": [],
            "docs": ["foobar"],
            "default": false
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
        .returns(ReturnTypeSpec::new(TypeSpec::with_name_str::<
            ink_primitives::ConstructorResult<()>,
        >(
            "ink_primitives::ConstructorResult"
        )))
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
            "returnType": {
                "displayName": [
                    "ink_primitives",
                    "ConstructorResult"
                ],
                "type": 0
            },
            "selector": "0x075bcd15",
            "args": [],
            "docs": [
                "Example",
                "```",
                "fn test() {",
                "    \"Hello, World\"",
                "}",
                "```"
            ],
            "default": false
        })
    );
    assert_eq!(deserialized.docs, compact_spec.docs);
}

#[test]
fn should_trim_whitespaces_in_events_docs() {
    // given
    let path: Path<PortableForm> =
        Path::from_segments_unchecked(["FooBarEvent".to_string()]);
    let spec = TypeSpec::new(789.into(), path);
    let args = [EventParamSpec::new("something".into())
        .of_type(spec)
        .indexed(true)
        .docs(["test"])
        .done()];
    let es = EventSpec::new("foobar".into())
        .module_path("foo")
        .signature_topic(Some([0u8; 32]))
        .args(args)
        .docs([" FooBarEvent  "])
        .done();

    let event_spec_name = serde_json::to_value(es).unwrap();

    // when
    let expected_event_spec = serde_json::json!(
        {
            "module_path": "foo",
            "signature_topic": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "args": [
            {
                "docs": ["test"],
                "indexed": true,
                "label": "something",
                "type": {
                    "displayName": [
                        "FooBarEvent"
                    ],
                    "type": 789
                }
            }],
            "docs": [
                "FooBarEvent"
            ],
            "label": "foobar"
        }
    );

    // then
    assert_eq!(event_spec_name, expected_event_spec);
}

/// Create a default environment spec with the `max_event_topics` set to `4`.
fn environment_spec() -> EnvironmentSpec<PortableForm> {
    EnvironmentSpec::new()
        .account_id(Default::default())
        .balance(Default::default())
        .hash(Default::default())
        .timestamp(Default::default())
        .block_number(Default::default())
        .static_buffer_size(16384)
        .native_to_eth_ratio(100_000_000)
        .done()
}

/// Helper for creating a constructor spec at runtime
fn runtime_constructor_spec() -> ConstructorSpec<PortableForm> {
    let path: Path<PortableForm> = Path::from_segments_unchecked(["FooType".to_string()]);
    let lang_path: Path<PortableForm> = Path::from_segments_unchecked([
        "ink_primitives".to_string(),
        "ConstructorResult".to_string(),
    ]);
    let spec = TypeSpec::new(123.into(), path);
    let ret_spec = ReturnTypeSpec::new(TypeSpec::new(456.into(), lang_path));
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
        .signature_topic(Some([42u8; 32]))
        .module_path("foo")
        .args(args)
        .docs(["foobar event"])
        .done()
}

/// Ensures constructing a `PortableForm` contract spec works at runtime
#[test]
fn construct_runtime_contract_spec() {
    let spec = ContractSpec::new()
        .environment(environment_spec())
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
            "returnType": {
                "displayName": [
                    "ink_primitives",
                    "ConstructorResult"
                ],
                "type": 456
            },
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
            ],
            "default": false
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
            "default": false,
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
            "module_path": "foo",
            "signature_topic": "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a",
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
