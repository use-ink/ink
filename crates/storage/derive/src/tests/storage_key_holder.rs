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

use crate::storage_key_holder_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageKeyHolder for UnitStruct {
                    const KEY: ::ink_primitives::StorageKey = <() as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn unit_struct_generic_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            struct UnitStruct<T>;
        }
        expands to {
            const _: () = {
                impl<T> ::ink_storage::traits::StorageKeyHolder for UnitStruct<T> {
                    const KEY: ::ink_primitives::StorageKey = <() as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn unit_struct_salt_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            struct UnitStruct<Salt: ::ink_storage::traits::StorageKeyHolder>;
        }
        expands to {
            const _: () = {
                impl<Salt: ::ink_storage::traits::StorageKeyHolder> ::ink_storage::traits::StorageKeyHolder for UnitStruct<Salt> {
                    const KEY: ::ink_primitives::StorageKey = <Salt as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageKeyHolder for NamedFields {
                    const KEY: ::ink_primitives::StorageKey = <() as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_generic_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            struct NamedFields<T> {
                a: T,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl<T> ::ink_storage::traits::StorageKeyHolder for NamedFields<T> {
                    const KEY: ::ink_primitives::StorageKey = <() as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_salt_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            struct NamedFields<Salt: StorageKeyHolder> {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl<Salt: StorageKeyHolder> ::ink_storage::traits::StorageKeyHolder for NamedFields<Salt> {
                    const KEY: ::ink_primitives::StorageKey = <Salt as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::StorageKeyHolder for MixedEnum {
                    const KEY: ::ink_primitives::StorageKey = <() as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_generic_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            enum MixedEnum<T> {
                A,
                B(T, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl<T> ::ink_storage::traits::StorageKeyHolder for MixedEnum<T> {
                    const KEY: ::ink_primitives::StorageKey = <() as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_salt_works() {
    crate::test_derive! {
        storage_key_holder_derive {
            enum MixedEnum<Salt: traits::StorageKeyHolder> {
                A,
                B(u32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl<Salt: traits::StorageKeyHolder> ::ink_storage::traits::StorageKeyHolder for MixedEnum<Salt> {
                    const KEY: ::ink_primitives::StorageKey = <Salt as ::ink_storage::traits::StorageKeyHolder>::KEY;
                }
            };
        }
        no_build
    }
}
