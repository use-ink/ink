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

use crate::packed_layout_derive;

#[test]
fn unit_struct_works() {
    synstructure::test_derive! {
        packed_layout_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::PackedLayout for UnitStruct {
                    fn pull_packed(&mut self, __key: &::ink_primitives::Key) {
                        match self {
                            UnitStruct => {}
                        }
                    }

                    fn push_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            UnitStruct => {}
                        }
                    }

                    fn clear_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            UnitStruct => {}
                        }
                    }
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_works() {
    synstructure::test_derive! {
        packed_layout_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::PackedLayout for NamedFields {
                    fn pull_packed(&mut self, __key: &::ink_primitives::Key) {
                        match self {
                            NamedFields {
                                a: __binding_0,
                                b: __binding_1,
                                d: __binding_2,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_1, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_2, __key);
                                }
                            }
                        }
                    }
                    fn push_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            NamedFields {
                                a: __binding_0,
                                b: __binding_1,
                                d: __binding_2,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_1, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_2, __key);
                                }
                            }
                        }
                    }
                    fn clear_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            NamedFields {
                                a: __binding_0,
                                b: __binding_1,
                                d: __binding_2,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_1, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_2, __key);
                                }
                            }
                        }
                    }
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_works() {
    synstructure::test_derive! {
        packed_layout_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::PackedLayout for MixedEnum {
                    fn pull_packed(&mut self, __key: &::ink_primitives::Key) {
                        match self {
                            MixedEnum::A => {}
                            MixedEnum::B(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_1, __key);
                                }
                            }
                            MixedEnum::C {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                    fn push_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            MixedEnum::A => {}
                            MixedEnum::B(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_1, __key);
                                }
                            }
                            MixedEnum::C {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                    fn clear_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            MixedEnum::A => {}
                            MixedEnum::B(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_1, __key);
                                }
                            }
                            MixedEnum::C {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                }
            };
        }
        no_build
    }
}

#[test]
fn generic_struct_works() {
    synstructure::test_derive! {
        packed_layout_derive {
            struct GenericStruct<T1, T2> {
                a: T1,
                b: (T1, T2),
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::PackedLayout for GenericStruct<T1, T2>
                where
                    T1: ::ink_storage::traits::PackedLayout,
                    T2: ::ink_storage::traits::PackedLayout
                {
                    fn pull_packed(&mut self, __key: &::ink_primitives::Key) {
                        match self {
                            GenericStruct {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                    fn push_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            GenericStruct {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                    fn clear_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            GenericStruct {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                }
            };
        }
        no_build
    }
}

#[test]
fn generic_enum_works() {
    synstructure::test_derive! {
        packed_layout_derive {
            enum GenericEnum<T1, T2> {
                Tuple(T1, T2),
                Named { a: T1, b: T2 },
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::PackedLayout for GenericEnum<T1, T2>
                where
                    T1: ::ink_storage::traits::PackedLayout,
                    T2: ::ink_storage::traits::PackedLayout
                {
                    fn pull_packed(&mut self, __key: &::ink_primitives::Key) {
                        match self {
                            GenericEnum::Tuple(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_1, __key);
                                }
                            }
                            GenericEnum::Named {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::pull_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                    fn push_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            GenericEnum::Tuple(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_1, __key);
                                }
                            }
                            GenericEnum::Named {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::push_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                    fn clear_packed(&self, __key: &::ink_primitives::Key) {
                        match self {
                            GenericEnum::Tuple(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_1, __key);
                                }
                            }
                            GenericEnum::Named {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_0, __key);
                                }
                                {
                                    ::ink_storage::traits::PackedLayout::clear_packed(__binding_1, __key);
                                }
                            }
                        }
                    }
                }
            };
        }
        no_build
    }
}
