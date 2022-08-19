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

use crate::storable_hint_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        storable_hint_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl<__ink_generic_salt: ::ink_storage::traits::StorageKey>
                    ::ink_storage::traits::StorableHint<__ink_generic_salt> for UnitStruct
                {
                    type Type = UnitStruct;
                    type PreferredKey = ::ink_storage::traits::AutoKey;
                }
            };
        }
        no_build
    }
}

#[test]
fn unit_struct_salt_works() {
    crate::test_derive! {
        storable_hint_derive {
            struct UnitStruct<Salt: ::ink_storage::traits::StorageKey>;
        }
        expands to {
            const _: () = {
                impl<
                        Salt: ::ink_storage::traits::StorageKey,
                        __ink_generic_salt: ::ink_storage::traits::StorageKey
                    >
                    ::ink_storage::traits::StorableHint<__ink_generic_salt> for UnitStruct<Salt>
                {
                    type Type = UnitStruct<__ink_generic_salt>;
                    type PreferredKey = Salt;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_works() {
    crate::test_derive! {
        storable_hint_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl<__ink_generic_salt: ::ink_storage::traits::StorageKey>
                    ::ink_storage::traits::StorableHint<__ink_generic_salt> for NamedFields
                {
                    type Type = NamedFields;
                    type PreferredKey = ::ink_storage::traits::AutoKey;
                }
            };
        }
        no_build
    }
}

#[test]
fn struct_salt_works() {
    crate::test_derive! {
        storable_hint_derive {
            struct NamedFields<Salt: StorageKey> {
                a: i32,
                b: [u8; 32],
                d: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl<
                        Salt: StorageKey,
                        __ink_generic_salt: ::ink_storage::traits::StorageKey
                    >
                    ::ink_storage::traits::StorableHint<__ink_generic_salt> for NamedFields<Salt>
                {
                    type Type = NamedFields<__ink_generic_salt>;
                    type PreferredKey = Salt;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_works() {
    crate::test_derive! {
        storable_hint_derive {
            enum MixedEnum {
                A,
                B(i32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl<__ink_generic_salt: ::ink_storage::traits::StorageKey>
                    ::ink_storage::traits::StorableHint<__ink_generic_salt> for MixedEnum
                {
                    type Type = MixedEnum;
                    type PreferredKey = ::ink_storage::traits::AutoKey;
                }
            };
        }
        no_build
    }
}

#[test]
fn enum_salt_works() {
    crate::test_derive! {
        storable_hint_derive {
            enum MixedEnum<Salt: traits::StorageKey> {
                A,
                B(u32, [u8; 32]),
                C { a: i32, b: (bool, i32) },
            }
        }
        expands to {
            const _: () = {
                impl<
                        Salt: traits::StorageKey,
                        __ink_generic_salt: ::ink_storage::traits::StorageKey
                    >
                    ::ink_storage::traits::StorableHint<__ink_generic_salt> for MixedEnum<Salt>
                {
                    type Type = MixedEnum<__ink_generic_salt>;
                    type PreferredKey = Salt;
                }
            };
        }
        no_build
    }
}
