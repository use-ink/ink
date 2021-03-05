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

use super::*;
use ink_primitives::KeyPtr;
use pretty_assertions::assert_eq;

#[test]
fn layout_key_works() {
    let layout_key = LayoutKey::from(Key::from([0x01; 32]));
    let json = serde_json::to_string(&layout_key).unwrap();
    assert_eq!(
        json,
        "\"0x0101010101010101010101010101010101010101010101010101010101010101\"",
    );
}

fn named_fields_struct_layout(key_ptr: &mut KeyPtr) -> Layout {
    StructLayout::new(vec![
        FieldLayout::new(
            "a",
            CellLayout::new::<i32>(LayoutKey::from(key_ptr.advance_by(1))),
        ),
        FieldLayout::new(
            "b",
            CellLayout::new::<i64>(LayoutKey::from(key_ptr.advance_by(1))),
        ),
    ])
    .into()
}

#[test]
fn named_fields_work() {
    let layout = named_fields_struct_layout(&mut KeyPtr::from(Key::from([0x00; 32])));
    let mut registry = Registry::new();
    let compacted = layout.into_portable(&mut registry);
    let json = serde_json::to_value(&compacted).unwrap();
    let expected = serde_json::json! {
        {
            "struct": {
                "fields": [
                    {
                        "layout": {
                            "cell": {
                                "key": "0x\
                                    0000000000000000\
                                    0000000000000000\
                                    0000000000000000\
                                    0000000000000000",
                                "ty": 1,
                            }
                        },
                        "name": "a",
                    },
                    {
                        "layout": {
                            "cell": {
                                "key": "0x\
                                    0100000000000000\
                                    0000000000000000\
                                    0000000000000000\
                                    0000000000000000",
                                "ty": 2,
                            }
                        },
                        "name": "b",
                    }
                ]
            }
        }
    };
    assert_eq!(json, expected);
}

fn tuple_struct_layout(key_ptr: &mut KeyPtr) -> Layout {
    StructLayout::new(vec![
        FieldLayout::new(
            None,
            CellLayout::new::<i32>(LayoutKey::from(key_ptr.advance_by(1))),
        ),
        FieldLayout::new(
            None,
            CellLayout::new::<i64>(LayoutKey::from(key_ptr.advance_by(1))),
        ),
    ])
    .into()
}

#[test]
fn tuple_struct_work() {
    let layout = tuple_struct_layout(&mut KeyPtr::from(Key::from([0x00; 32])));
    let mut registry = Registry::new();
    let compacted = layout.into_portable(&mut registry);
    let json = serde_json::to_value(&compacted).unwrap();
    let expected = serde_json::json! {
        {
            "struct": {
                "fields": [
                    {
                        "layout": {
                            "cell": {
                                "key": "0x\
                                    0000000000000000\
                                    0000000000000000\
                                    0000000000000000\
                                    0000000000000000",
                                "ty": 1,
                            }
                        },
                        "name": null,
                    },
                    {
                        "layout": {
                            "cell": {
                                "key": "0x\
                                    0100000000000000\
                                    0000000000000000\
                                    0000000000000000\
                                    0000000000000000",
                                "ty": 2,
                            }
                        },
                        "name": null,
                    }
                ]
            }
        }
    };
    assert_eq!(json, expected);
}

fn clike_enum_layout(key_ptr: &mut KeyPtr) -> Layout {
    EnumLayout::new(
        key_ptr.advance_by(1),
        vec![
            (Discriminant(0), StructLayout::new(vec![])),
            (Discriminant(1), StructLayout::new(vec![])),
            (Discriminant(2), StructLayout::new(vec![])),
        ],
    )
    .into()
}

#[test]
fn clike_enum_work() {
    let layout = clike_enum_layout(&mut KeyPtr::from(Key::from([0x00; 32])));
    let mut registry = Registry::new();
    let compacted = layout.into_portable(&mut registry);
    let json = serde_json::to_value(&compacted).unwrap();
    let expected = serde_json::json! {
        {
            "enum": {
                "dispatchKey": "0x\
                    0000000000000000\
                    0000000000000000\
                    0000000000000000\
                    0000000000000000",
                "variants": {
                    "0": {
                        "fields": [],
                    },
                    "1": {
                        "fields": [],
                    },
                    "2": {
                        "fields": [],
                    },
                }
            }
        }
    };
    assert_eq!(json, expected);
}

fn mixed_enum_layout(key_ptr: &mut KeyPtr) -> Layout {
    EnumLayout::new(
        *key_ptr.advance_by(1),
        vec![
            (Discriminant(0), StructLayout::new(vec![])),
            {
                let mut variant_key_ptr = key_ptr.clone();
                (
                    Discriminant(1),
                    StructLayout::new(vec![
                        FieldLayout::new(
                            None,
                            CellLayout::new::<i32>(LayoutKey::from(
                                variant_key_ptr.advance_by(1),
                            )),
                        ),
                        FieldLayout::new(
                            None,
                            CellLayout::new::<i64>(LayoutKey::from(
                                variant_key_ptr.advance_by(1),
                            )),
                        ),
                    ]),
                )
            },
            {
                let mut variant_key_ptr = key_ptr.clone();
                (
                    Discriminant(2),
                    StructLayout::new(vec![
                        FieldLayout::new(
                            "a",
                            CellLayout::new::<i32>(LayoutKey::from(
                                variant_key_ptr.advance_by(1),
                            )),
                        ),
                        FieldLayout::new(
                            "b",
                            CellLayout::new::<i64>(LayoutKey::from(
                                variant_key_ptr.advance_by(1),
                            )),
                        ),
                    ]),
                )
            },
        ],
    )
    .into()
}

#[test]
fn mixed_enum_work() {
    let layout = mixed_enum_layout(&mut KeyPtr::from(Key::from([0x00; 32])));
    let mut registry = Registry::new();
    let compacted = layout.into_portable(&mut registry);
    let json = serde_json::to_value(&compacted).unwrap();
    let expected = serde_json::json! {
        {
            "enum": {
                "dispatchKey": "0x\
                    0000000000000000\
                    0000000000000000\
                    0000000000000000\
                    0000000000000000",
                "variants": {
                    "0": {
                        "fields": [],
                    },
                    "1": {
                        "fields": [
                            {
                                "layout": {
                                    "cell": {
                                        "key": "0x\
                                            0100000000000000\
                                            0000000000000000\
                                            0000000000000000\
                                            0000000000000000",
                                        "ty": 1,
                                    }
                                },
                                "name": null,
                            },
                            {
                                "layout": {
                                    "cell": {
                                        "key": "0x\
                                            0200000000000000\
                                            0000000000000000\
                                            0000000000000000\
                                            0000000000000000",
                                        "ty": 2,
                                    }
                                },
                                "name": null,
                            }
                        ],
                    },
                    "2": {
                        "fields": [
                            {
                                "layout": {
                                    "cell": {
                                        "key": "0x\
                                            0100000000000000\
                                            0000000000000000\
                                            0000000000000000\
                                            0000000000000000",
                                        "ty": 1,
                                    }
                                },
                                "name": "a",
                            },
                            {
                                "layout": {
                                    "cell": {
                                        "key": "0x\
                                            0200000000000000\
                                            0000000000000000\
                                            0000000000000000\
                                            0000000000000000",
                                        "ty": 2,
                                    }
                                },
                                "name": "b",
                            }
                        ],
                    },
                }
            }
        }
    };
    assert_eq!(json, expected);
}

fn unbounded_hashing_layout(key_ptr: &mut KeyPtr) -> Layout {
    let root_key = key_ptr.advance_by(1);
    HashLayout::new(
        root_key,
        HashingStrategy::new(
            CryptoHasher::Blake2x256,
            b"ink storage hashmap".to_vec(),
            Vec::new(),
        ),
        CellLayout::new::<(i32, bool)>(LayoutKey::from(root_key)),
    )
    .into()
}

#[test]
fn unbounded_layout_works() {
    let layout = unbounded_hashing_layout(&mut KeyPtr::from(Key::from([0x00; 32])));
    let mut registry = Registry::new();
    let compacted = layout.into_portable(&mut registry);
    let json = serde_json::to_value(&compacted).unwrap();
    let expected = serde_json::json! {
        {
            "hash": {
                "layout": {
                    "cell": {
                        "key": "0x\
                            0000000000000000\
                            0000000000000000\
                            0000000000000000\
                            0000000000000000",
                        "ty": 1
                    }
                },
                "offset": "0x\
                    0000000000000000\
                    0000000000000000\
                    0000000000000000\
                    0000000000000000",
                "strategy": {
                        "hasher": "Blake2x256",
                        "prefix": "0x696e6b2073746f7261676520686173686d6170",
                        "postfix": "",
                }
            }
        }
    };
    assert_eq!(json, expected);
}
