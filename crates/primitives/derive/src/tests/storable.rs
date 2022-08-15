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

// These tests are partly testing if code is expanded correctly.
// Hence the syntax contains a number of verbose statements which
// are not properly cleaned up.
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::identity_op)]
#![allow(clippy::eq_op)]
#![allow(clippy::match_single_binding)]

use crate::storable_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        storable_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_primitives::traits::Storable for UnitStruct {
                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(UnitStruct)
                    }

                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                        match self {
                            UnitStruct => { }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn struct_works() {
    crate::test_derive! {
        storable_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_primitives::traits::Storable for NamedFields {
                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(
                            NamedFields {
                                a : <i32 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                b : <[u8; 32] as ::ink_primitives::traits::Storable>::decode(__input)?,
                                d : <Box<i32> as ::ink_primitives::traits::Storable>::decode(__input)?,
                            }
                        )
                    }

                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                        match self {

                            NamedFields {
                                a: __binding_0,
                                b: __binding_1,
                                d: __binding_2,
                            } => {
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_0,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_1,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_2,
                                        __dest
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
fn one_variant_enum_works() {
    crate::test_derive! {
        storable_derive {
            enum OneVariantEnum {
                A,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_primitives::traits::Storable for OneVariantEnum {
                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(
                            match <::core::primitive::u8 as ::ink_primitives::traits::Storable>::decode(__input)?
                            {
                                0u8 => OneVariantEnum::A,
                                _ => unreachable!("encountered invalid enum discriminant"),
                            }
                        )
                    }

                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                        match self {
                            OneVariantEnum::A => {
                                {
                                    <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(
                                        &0u8,
                                        __dest
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
    crate::test_derive! {
        storable_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl ::ink_primitives::traits::Storable for MixedEnum {
                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(
                            match <::core::primitive::u8 as ::ink_primitives::traits::Storable>::decode(__input)?
                            {
                                0u8 => MixedEnum::A,
                                1u8 => MixedEnum::B(
                                    <i32 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                    <[u8; 32] as ::ink_primitives::traits::Storable>::decode(__input)?,
                                ),
                                2u8 => MixedEnum::C {
                                    a: < i32 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                    b: <(bool, i32) as ::ink_primitives::traits::Storable>::decode(__input)?,
                                },
                                _ => unreachable!("encountered invalid enum discriminant"),
                            }
                        )
                    }

                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                        match self {
                            MixedEnum::A => {
                                {
                                    <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(
                                        &0u8,
                                        __dest
                                    );
                                }
                            }
                            MixedEnum::B(__binding_0, __binding_1,) => {
                                {
                                    <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(
                                        &1u8,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_0,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_1,
                                        __dest
                                    );
                                }
                            }
                            MixedEnum::C {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(
                                        &2u8, __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_0,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_1,
                                        __dest
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
    crate::test_derive! {
        storable_derive {
            struct GenericStruct<T1, T2>
            where
                T1: ::scale::Decode,
                T2: ::scale::Encode,
            {
                a: T1,
                b: (T1, T2),
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_primitives::traits::Storable for GenericStruct<T1, T2>
                where
                    T1: ::scale::Decode,
                    T2: ::scale::Encode,
                    T1: ::ink_primitives::traits::Storable,
                    (T1 , T2): ::ink_primitives::traits::Storable
                {
                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(
                            GenericStruct {
                                a: <T1 as ::ink_primitives::traits::Storable>::decode(
                                    __input
                                )?,
                                b: <(T1, T2) as ::ink_primitives::traits::Storable>::decode(
                                    __input
                                )?,
                            }
                        )
                    }

                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                        match self {
                            GenericStruct {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_0,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_1,
                                        __dest
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
    crate::test_derive! {
        storable_derive {
            enum GenericEnum<T1, T2> {
                Tuple(T1, T2),
                Named { a: T1, b: T2 },
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_primitives::traits::Storable for GenericEnum<T1, T2>
                where
                    T1: ::ink_primitives::traits::Storable,
                    T2: ::ink_primitives::traits::Storable
                {
                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(
                            match <::core::primitive::u8 as ::ink_primitives::traits::Storable>::decode(__input)?
                            {
                                0u8 => GenericEnum::Tuple(
                                    <T1 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                    <T2 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                ),
                                1u8 => GenericEnum::Named {
                                    a: <T1 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                    b: <T2 as ::ink_primitives::traits::Storable>::decode(__input)?,
                                },
                                _ => unreachable!("encountered invalid enum discriminant"),
                            }
                        )
                    }

                    #[inline(always)]
                    #[allow(non_camel_case_types)]
                    fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                        match self {
                            GenericEnum::Tuple(__binding_0, __binding_1,) => {
                                {
                                    <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(&0u8, __dest);
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_0,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_1,
                                        __dest
                                    );
                                }
                            }
                            GenericEnum::Named {
                                a: __binding_0,
                                b: __binding_1,
                            } => {
                                {
                                    <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(&1u8, __dest);
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_0,
                                        __dest
                                    );
                                }
                                {
                                    ::ink_primitives::traits::Storable::encode(
                                        __binding_1,
                                        __dest
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
