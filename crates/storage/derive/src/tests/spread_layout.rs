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

use crate::spread_layout_derive;

#[test]
fn unit_struct_works() {
    synstructure::test_derive! {
        spread_layout_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::SpreadLayout for UnitStruct {
                    #[allow(unused_comparisons)]
                    const FOOTPRINT: u64 = [0u64, 0u64][(0u64 < 0u64) as usize];

                    const REQUIRES_DEEP_CLEAN_UP : bool = (false || false );

                    fn pull_spread(__key_ptr: &mut ::ink_storage::traits::KeyPtr) -> Self {
                        UnitStruct
                    }

                    fn push_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            UnitStruct => {}
                        }
                    }

                    fn clear_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            UnitStruct => {}
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn struct_works() {
    synstructure::test_derive! {
        spread_layout_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::SpreadLayout for NamedFields {
                    #[allow(unused_comparisons)]
                    const FOOTPRINT: u64 = [
                        (((0u64 + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            + <[u8; 32] as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            + <Box<i32> as ::ink_storage::traits::SpreadLayout>::FOOTPRINT),
                        0u64
                    ][((((0u64
                        + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        + <[u8; 32] as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        + <Box<i32> as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        < 0u64) as usize
                    ];

                    const REQUIRES_DEEP_CLEAN_UP : bool = (
                        false || (
                            (
                                (
                                    false
                                    || <i32 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                                )
                                || <[u8; 32] as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                            )
                            || <Box<i32> as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                        )
                    );

                    fn pull_spread(__key_ptr: &mut ::ink_storage::traits::KeyPtr) -> Self {
                        NamedFields {
                            a : <i32 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                            b : <[u8; 32] as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                            d : <Box<i32> as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                        }
                    }

                    fn push_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            NamedFields {
                                a: __binding_0,
                                b: __binding_1,
                                d: __binding_2,
                            } => {
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_2,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }

                    fn clear_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            NamedFields {
                                a: __binding_0,
                                b: __binding_1,
                                d: __binding_2,
                            } => {
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_2,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn enum_works() {
    synstructure::test_derive! {
        spread_layout_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::SpreadLayout for MixedEnum {
                    #[allow(unused_comparisons)]
                    const FOOTPRINT : u64 = 1 + [
                        0u64 ,
                        [
                            (
                                (
                                    0u64
                                    + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                )
                                + <[u8; 32] as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            ,
                            [
                                (
                                    (
                                        0u64
                                        + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    )
                                    + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                ),
                                0u64
                            ]
                            [
                                (
                                    (
                                        (
                                            0u64
                                            + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        )
                                        + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    )
                                    < 0u64
                                ) as usize
                            ]
                        ][
                            (
                                (
                                    (
                                        0u64
                                        + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    )
                                    + <[u8; 32] as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                )
                                <[
                                    (
                                        (
                                            0u64
                                            + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        )
                                        + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    ),
                                    0u64
                                ][
                                    (
                                        (
                                            (
                                                0u64
                                                + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                            )
                                            + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        )
                                        < 0u64
                                    ) as usize
                                ]
                            ) as usize
                        ]
                    ][
                        (
                            0u64 <[
                                (
                                    (
                                        0u64
                                        + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    )
                                    + <[u8; 32] as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                ),
                                [
                                    (
                                        (
                                            0u64
                                            + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        )
                                        + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    ),
                                    0u64
                                ][
                                    (
                                        (
                                            (
                                                0u64
                                                + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                            )
                                            + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        )
                                        < 0u64
                                    ) as usize
                                ]
                            ][
                                (
                                    (
                                        (
                                            0u64
                                            + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        )
                                        + <[u8; 32] as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                    )
                                    <[
                                        (
                                            (
                                                0u64
                                                + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                            )
                                            + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                        ),
                                        0u64
                                    ][
                                        (
                                            (
                                                (
                                                    0u64
                                                    + <i32 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                                )
                                                + <(bool, i32) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT
                                            )
                                            < 0u64
                                        ) as usize
                                    ]
                                ) as usize
                            ]
                        ) as usize
                    ];

                    const REQUIRES_DEEP_CLEAN_UP : bool = (
                        (
                            (false || false)
                            || (
                                (
                                    false
                                    || <i32 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                                )
                                || <[u8; 32] as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                            )
                        )
                        || (
                            (
                                false
                                || <i32 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                            )
                            || <(bool, i32) as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                        )
                    );

                    fn pull_spread(__key_ptr: &mut ::ink_storage::traits::KeyPtr) -> Self {
                        match <u8 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr)
                        {
                            0u8 => MixedEnum::A,
                            1u8 => MixedEnum::B(
                                <i32 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                                <[u8; 32] as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                            ),
                            2u8 => MixedEnum::C {
                                a: < i32 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr ),
                                b: <(bool, i32) as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                            },
                            _ => unreachable!("encountered invalid enum discriminant"),
                        }
                    }
                    fn push_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            MixedEnum::A => {
                                {
                                    <u8 as ::ink_storage::traits::SpreadLayout>::push_spread(
                                        &0u8,
                                        __key_ptr
                                    );
                                }
                            }
                            MixedEnum::B(__binding_0, __binding_1,) => {
                                {
                                    <u8 as ::ink_storage::traits::SpreadLayout>::push_spread(
                                        &1u8,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                            MixedEnum::C {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    <u8 as ::ink_storage::traits::SpreadLayout>::push_spread(
                                        &2u8, __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }
                    fn clear_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            MixedEnum::A => {}
                            MixedEnum::B(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                            MixedEnum::C {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn generic_struct_works() {
    synstructure::test_derive! {
        spread_layout_derive {
            struct GenericStruct<T1, T2> {
                a: T1,
                b: (T1, T2),
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::SpreadLayout for GenericStruct<T1, T2>
                where
                    T1: ::ink_storage::traits::SpreadLayout,
                    T2: ::ink_storage::traits::SpreadLayout
                {
                    #[allow(unused_comparisons)]
                    const FOOTPRINT: u64 = [
                        ((0u64 + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            + <(T1, T2) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT),
                        0u64
                    ][(((0u64 + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        + <(T1, T2) as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        < 0u64) as usize
                    ];

                    const REQUIRES_DEEP_CLEAN_UP : bool = (
                        false || (
                            (
                                false
                                || <T1 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                            )
                            || < (T1, T2) as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                        )
                    );

                    fn pull_spread(__key_ptr: &mut ::ink_storage::traits::KeyPtr) -> Self {
                        GenericStruct {
                            a: <T1 as ::ink_storage::traits::SpreadLayout>::pull_spread(
                                __key_ptr
                            ),
                            b: <(T1, T2) as ::ink_storage::traits::SpreadLayout>::pull_spread(
                                __key_ptr
                            ),
                        }
                    }

                    fn push_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            GenericStruct {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }

                    fn clear_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            GenericStruct {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn generic_enum_works() {
    synstructure::test_derive! {
        spread_layout_derive {
            enum GenericEnum<T1, T2> {
                Tuple(T1, T2),
                Named { a: T1, b: T2 },
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::SpreadLayout for GenericEnum<T1, T2>
                where
                    T1: ::ink_storage::traits::SpreadLayout,
                    T2: ::ink_storage::traits::SpreadLayout
                {
                    #[allow(unused_comparisons)]
                    const FOOTPRINT: u64 = 1 + [
                        ((0u64 + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            + <T2 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT),
                        [
                            ((0u64 + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                                + <T2 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT),
                            0u64
                        ][(((0u64
                            + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            + <T2 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            < 0u64) as usize]
                    ][(((0u64 + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        + <T2 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                        < [
                            ((0u64 + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                                + <T2 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT),
                            0u64
                        ][(((0u64
                            + <T1 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            + <T2 as ::ink_storage::traits::SpreadLayout>::FOOTPRINT)
                            < 0u64) as usize]) as usize
                    ];

                    const REQUIRES_DEEP_CLEAN_UP : bool = (
                        (
                            false || (
                                (
                                    false
                                    || <T1 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                                )
                                || <T2 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                            )
                        )
                        || (
                            (
                                false
                                || <T1 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                            )
                            || <T2 as ::ink_storage::traits::SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
                        )
                    );

                    fn pull_spread(__key_ptr: &mut ::ink_storage::traits::KeyPtr) -> Self {
                        match <u8 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr)
                        {
                            0u8 => GenericEnum::Tuple(
                                <T1 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                                <T2 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                            ),
                            1u8 => GenericEnum::Named {
                                a: <T1 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                                b: <T2 as ::ink_storage::traits::SpreadLayout>::pull_spread(__key_ptr),
                            },
                            _ => unreachable!("encountered invalid enum discriminant"),
                        }
                    }

                    fn push_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            GenericEnum::Tuple(__binding_0, __binding_1,) => {
                                {
                                    <u8 as ::ink_storage::traits::SpreadLayout>::push_spread(&0u8, __key_ptr);
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                            GenericEnum::Named {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    <u8 as ::ink_storage::traits::SpreadLayout>::push_spread(&1u8, __key_ptr);
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::push_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }

                    fn clear_spread(&self, __key_ptr: &mut ::ink_storage::traits::KeyPtr) {
                        match self {
                            GenericEnum::Tuple(__binding_0, __binding_1,) => {
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                            GenericEnum::Named {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_0,
                                        __key_ptr
                                    );
                                }
                                {
                                    ::ink_storage::traits::SpreadLayout::clear_spread(
                                        __binding_1,
                                        __key_ptr
                                    );
                                }
                            }
                        }
                    }
                }
            };
        }
    }
}
