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

#![cfg_attr(not(feature = "std"), no_std)]
#[ink::contract]
mod contract {
    #[ink::trait_definition]
    pub trait TraitDefinition {
        #[ink(message)]
        fn get_value(&self) -> u32;
    }

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
    }

    impl TraitDefinition for Contract {
        #[ink(message)]
        fn get_value(&self) -> u32 {
            42
        }
    }

    #[cfg(test)]
    mod tests {
        #[ink::test]
        fn foo() {
            assert!(true)
        }
    }
}

#[cfg(test)]
mod tests {
    fn generate_metadata() -> ink_metadata::InkProject {
        extern "Rust" {
            fn __ink_generate_metadata() -> ink_metadata::InkProject;
        }

        unsafe { __ink_generate_metadata() }
    }

    #[test]
    fn trait_message_return_value_is_result() {
        let metadata = generate_metadata();

        let message = metadata.spec().messages().iter().next().unwrap();
        assert_eq!("TraitDefinition::get_value", message.label());

        let type_spec = message.return_type().opt_type().unwrap();
        let ty = metadata.registry().resolve(type_spec.ty().id()).unwrap();
        assert_eq!(
            "Result",
            format!("{}", ty.path()),
            "Message return type should be a Result"
        );
        match ty.type_def() {
            scale_info::TypeDef::Variant(variant) => {
                assert_eq!(2, variant.variants().len());
                let ok_variant = &variant.variants()[0];
                let ok_field = &ok_variant.fields()[0];
                let ok_ty = metadata.registry().resolve(ok_field.ty().id()).unwrap();

                assert_eq!("Ok", ok_variant.name());
                assert_eq!(
                    &scale_info::TypeDef::Primitive(scale_info::TypeDefPrimitive::U32),
                    ok_ty.type_def()
                )
            }
            td => panic!("Expected a Variant type def enum, got {:?}", td),
        }
    }
}
