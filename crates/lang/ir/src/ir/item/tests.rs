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

#[test]
fn simple_storage_works() {
    let storage_struct: syn::Item = syn::parse_quote! {
        #[ink(storage)]
        pub struct MyStorage {
            field_1: bool,
            field_2: i32,
        }
    };
    assert!(matches!(
        <ir::Item as TryFrom<_>>::try_from(storage_struct.clone())
            .map_err(|err| err.to_string()),
        Ok(ir::Item::Ink(ir::InkItem::Storage(_)))
    ))
}

#[test]
fn simple_event_works() {
    let event_struct: syn::Item = syn::parse_quote! {
        #[ink(event)]
        pub struct MyEvent {
            #[ink(topic)]
            param_1: bool,
            param_2: i32,
        }
    };
    assert!(matches!(
        <ir::Item as TryFrom<_>>::try_from(event_struct.clone())
            .map_err(|err| err.to_string()),
        Ok(ir::Item::Ink(ir::InkItem::Event(_)))
    ))
}

#[test]
fn simple_rust_item_works() {
    let rust_items: Vec<syn::Item> = vec![
        syn::parse_quote! {
            struct RustStruct {
                field_1: bool,
                field_2: i32,
            }
        },
        syn::parse_quote! {
            enum RustEnum {
                Variant1,
                Variant2(bool),
                Variant3 {
                    a: i32,
                    b: i32,
                }
            }
        },
        syn::parse_quote! {
            fn rust_function(param1: bool, param2: i32) {}
        },
        syn::parse_quote! {
            mod rust_module {}
        },
    ];
    for rust_item in rust_items {
        assert_eq!(
            <ir::Item as TryFrom<_>>::try_from(rust_item.clone())
                .map_err(|err| err.to_string()),
            Ok(ir::Item::Rust(rust_item))
        )
    }
}
