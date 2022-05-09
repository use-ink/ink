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

use crate::atomic_guard_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        atomic_guard_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::AtomicGuard<true> for UnitStruct {}
            };
        }
        no_build
    }
}

#[test]
fn struct_works() {
    crate::test_derive! {
        atomic_guard_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::AtomicGuard<true> for NamedFields
                where
                    i32: ::ink_storage::traits::AtomicGuard<true>,
                    [u8; 32]: ::ink_storage::traits::AtomicGuard<true>,
                    Box<i32>: ::ink_storage::traits::AtomicGuard<true>
                {}
            };
        }
        no_build
    }
}

#[test]
fn enum_works() {
    crate::test_derive! {
        atomic_guard_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::AtomicGuard<true> for MixedEnum
                where
                    i32: ::ink_storage::traits::AtomicGuard<true>,
                    [u8; 32]: ::ink_storage::traits::AtomicGuard<true>,
                    (bool, i32): ::ink_storage::traits::AtomicGuard<true>
                {}
            };
        }
        no_build
    }
}

#[test]
fn generic_struct_works() {
    crate::test_derive! {
        atomic_guard_derive {
            struct GenericStruct<T1, T2> {
                a: T1,
                b: (T1, T2),
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::AtomicGuard<true> for GenericStruct<T1, T2>
                where
                    T1: ::ink_storage::traits::AtomicGuard<true>,
                    (T1, T2): ::ink_storage::traits::AtomicGuard<true>
                {}
            };
        }
        no_build
    }
}

#[test]
fn generic_enum_works() {
    crate::test_derive! {
        atomic_guard_derive {
            enum GenericEnum<T1, T2> {
                Tuple(T1, T2),
                Named { a: T1, b: T2 },
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::AtomicGuard<true> for GenericEnum<T1, T2>
                where
                    T1: ::ink_storage::traits::AtomicGuard<true>,
                    T2: ::ink_storage::traits::AtomicGuard<true>
                {}
            };
        }
        no_build
    }
}
