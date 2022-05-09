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

use crate::storage_layout_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        storage_layout_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageLayout for UnitStruct {
                    fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                        ::ink_metadata::layout::Layout::Struct(
                            ::ink_metadata::layout::StructLayout::new([])
                        )
                    }
                }
            };
        }
    }
}

#[test]
fn tuple_struct_works() {
    crate::test_derive! {
        storage_layout_derive {
            struct TupleStruct(bool, u32, i64);
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageLayout for TupleStruct {
                    fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                        ::ink_metadata::layout::Layout::Struct(
                            ::ink_metadata::layout::StructLayout::new([
                                ::ink_metadata::layout::FieldLayout::new(
                                    ::core::option::Option::None,
                                    <bool as ::ink_storage::traits::StorageLayout>::layout(__key),
                                ),
                                ::ink_metadata::layout::FieldLayout::new(
                                    ::core::option::Option::None,
                                    <u32 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                ),
                                ::ink_metadata::layout::FieldLayout::new(
                                    ::core::option::Option::None,
                                    <i64 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                ),
                            ])
                        )
                    }
                }
            };
        }
    }
}

#[test]
fn named_fields_struct_works() {
    crate::test_derive! {
        storage_layout_derive {
            struct NamedFieldsStruct {
                a: bool,
                b: u32,
                c: i64,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageLayout for NamedFieldsStruct {
                    fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                        ::ink_metadata::layout::Layout::Struct(
                            ::ink_metadata::layout::StructLayout::new([
                                ::ink_metadata::layout::FieldLayout::new(
                                    ::core::option::Option::Some("a"),
                                    <bool as ::ink_storage::traits::StorageLayout>::layout(__key),
                                ),
                                ::ink_metadata::layout::FieldLayout::new(
                                    ::core::option::Option::Some("b"),
                                    <u32 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                ),
                                ::ink_metadata::layout::FieldLayout::new(
                                    ::core::option::Option::Some("c"),
                                    <i64 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                ),
                            ])
                        )
                    }
                }
            };
        }
    }
}

#[test]
fn clike_enum_works() {
    crate::test_derive! {
        storage_layout_derive {
            enum ClikeEnum { A, B, C }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageLayout for ClikeEnum {
                    fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                        ::ink_metadata::layout::Layout::Enum(
                            ::ink_metadata::layout::EnumLayout::new(
                                ::ink_metadata::layout::LayoutKey::from(__key),
                                [
                                    {
                                        (
                                            ::ink_metadata::layout::Discriminant::from(0usize),
                                            ::ink_metadata::layout::StructLayout::new([]),
                                        )
                                    },
                                    {
                                        (
                                            ::ink_metadata::layout::Discriminant::from(1usize),
                                            ::ink_metadata::layout::StructLayout::new([]),
                                        )
                                    },
                                    {
                                        (
                                            ::ink_metadata::layout::Discriminant::from(2usize),
                                            ::ink_metadata::layout::StructLayout::new([]),
                                        )
                                    },
                                ]
                            )
                        )
                    }
                }
            };
        }
    }
}

#[test]
fn mixed_enum_works() {
    crate::test_derive! {
        storage_layout_derive {
            enum MixedEnum {
                A,
                B(bool, u32, i64),
                C{
                    a: bool,
                    b: u32,
                    c: i64,
                }
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageLayout for MixedEnum {
                    fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                        ::ink_metadata::layout::Layout::Enum(
                            ::ink_metadata::layout::EnumLayout::new(
                                ::ink_metadata::layout::LayoutKey::from(__key),
                                [
                                    {
                                        (
                                            ::ink_metadata::layout::Discriminant::from(0usize),
                                            ::ink_metadata::layout::StructLayout::new([]),
                                        )
                                    },
                                    {
                                        (
                                            ::ink_metadata::layout::Discriminant::from(1usize),
                                            ::ink_metadata::layout::StructLayout::new([
                                                ::ink_metadata::layout::FieldLayout::new(
                                                    ::core::option::Option::None,
                                                    <bool as ::ink_storage::traits::StorageLayout>::layout(__key),
                                                ),
                                                ::ink_metadata::layout::FieldLayout::new(
                                                    ::core::option::Option::None,
                                                    <u32 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                                ),
                                                ::ink_metadata::layout::FieldLayout::new(
                                                    ::core::option::Option::None,
                                                    <i64 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                                ),
                                            ]),
                                        )
                                    },
                                    {
                                        (
                                            ::ink_metadata::layout::Discriminant::from(2usize),
                                            ::ink_metadata::layout::StructLayout::new([
                                                ::ink_metadata::layout::FieldLayout::new(
                                                    ::core::option::Option::Some("a"),
                                                    <bool as ::ink_storage::traits::StorageLayout>::layout(__key),
                                                ),
                                                ::ink_metadata::layout::FieldLayout::new(
                                                    ::core::option::Option::Some("b"),
                                                    <u32 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                                ),
                                                ::ink_metadata::layout::FieldLayout::new(
                                                    ::core::option::Option::Some("c"),
                                                    <i64 as ::ink_storage::traits::StorageLayout>::layout(__key),
                                                ),
                                            ]),
                                        )
                                    },
                                ]
                            )
                        )
                    }
                }
            };
        }
    }
}
