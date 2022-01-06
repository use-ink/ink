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

use crate::spread_allocate_derive;
use syn::parse_quote;

#[test]
#[should_panic(expected = "cannot derive `SpreadAllocate` for `enum` types")]
fn enum_fails() {
    spread_allocate_derive(synstructure::Structure::new(&parse_quote! {
        enum Enum { A, B, C }
    }));
}

#[test]
#[should_panic(expected = "Unable to create synstructure::Structure: \
                           Error(\"unexpected unsupported untagged union\")")]
fn union_fails() {
    spread_allocate_derive(synstructure::Structure::new(&parse_quote! {
        union Union { a: i32, b: u32 }
    }));
}

#[test]
fn unit_struct_works() {
    synstructure::test_derive! {
        spread_allocate_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::SpreadAllocate for UnitStruct {
                    fn allocate_spread(__key_ptr: &mut ::ink_primitives::KeyPtr) -> Self {
                        UnitStruct
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
        spread_allocate_derive {
            struct NamedFields {
                a: i32,
                b: [u8; 32],
                c: Box<i32>,
            }
        }
        expands to {
            const _: () = {
                impl ::ink_storage::traits::SpreadAllocate for NamedFields {
                    fn allocate_spread(__key_ptr: &mut ::ink_primitives::KeyPtr) -> Self {
                        NamedFields {
                            a: <i32 as ::ink_storage::traits::SpreadAllocate>::allocate_spread(__key_ptr),
                            b: <[u8; 32] as ::ink_storage::traits::SpreadAllocate>::allocate_spread(__key_ptr),
                            c: <Box<i32> as ::ink_storage::traits::SpreadAllocate>::allocate_spread(__key_ptr),
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
        spread_allocate_derive {
            struct GenericStruct<T1, T2> {
                a: T1,
                b: (T1, T2),
            }
        }
        expands to {
            const _: () = {
                impl<T1, T2> ::ink_storage::traits::SpreadAllocate for GenericStruct<T1, T2>
                where
                    T1: ::ink_storage::traits::SpreadAllocate,
                    T2: ::ink_storage::traits::SpreadAllocate
                {
                    fn allocate_spread(__key_ptr: &mut ::ink_primitives::KeyPtr) -> Self {
                        GenericStruct {
                            a: <T1 as ::ink_storage::traits::SpreadAllocate>::allocate_spread(__key_ptr),
                            b: <(T1, T2) as ::ink_storage::traits::SpreadAllocate>::allocate_spread(__key_ptr),
                        }
                    }
                }
            };
        }
        no_build
    }
}
