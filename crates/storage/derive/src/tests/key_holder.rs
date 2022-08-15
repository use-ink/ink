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

use crate::key_holder_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        key_holder_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageKey for UnitStruct {
                    const KEY: ::ink_primitives::Key = <() as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn unit_struct_generic_works() {
    crate::test_derive! {
        key_holder_derive {
            struct UnitStruct<T>;
        }
        expands to {
            const _: () = {
                impl<T> ::ink_storage::traits::StorageKey for UnitStruct<T> {
                    const KEY: ::ink_primitives::Key = <() as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn unit_struct_salt_works() {
    crate::test_derive! {
        key_holder_derive {
            struct UnitStruct<Salt: ::ink_storage::traits::StorageKey>;
        }
        expands to {
            const _: () = {
                impl<Salt: ::ink_storage::traits::StorageKey> ::ink_storage::traits::StorageKey for UnitStruct<Salt> {
                    const KEY: ::ink_primitives::Key = <Salt as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_works() {
    crate::test_derive! {
        key_holder_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageKey for NamedFields {
                    const KEY: ::ink_primitives::Key = <() as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_generic_works() {
    crate::test_derive! {
        key_holder_derive {
            struct NamedFields<T> {
                a: T,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl<T> ::ink_storage::traits::StorageKey for NamedFields<T> {
                    const KEY: ::ink_primitives::Key = <() as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_salt_works() {
    crate::test_derive! {
        key_holder_derive {
            struct NamedFields<Salt: StorageKey> {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl<Salt: StorageKey> ::ink_storage::traits::StorageKey for NamedFields<Salt> {
                    const KEY: ::ink_primitives::Key = <Salt as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_works() {
    crate::test_derive! {
        key_holder_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageKey for MixedEnum {
                    const KEY: ::ink_primitives::Key = <() as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_generic_works() {
    crate::test_derive! {
        key_holder_derive {
            enum MixedEnum<T> {
                A,
                B(T, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl<T> ::ink_storage::traits::StorageKey for MixedEnum<T> {
                    const KEY: ::ink_primitives::Key = <() as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_salt_works() {
    crate::test_derive! {
        key_holder_derive {
            enum MixedEnum<Salt: traits::StorageKey> {
                A,
                B(u32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl<Salt: traits::StorageKey> ::ink_storage::traits::StorageKey for MixedEnum<Salt> {
                    const KEY: ::ink_primitives::Key = <Salt as ::ink_storage::traits::StorageKey>::KEY;
                }
            };
        }
        no_build
    }
}
