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
        pub fn try_new() -> Result<Self, u8> {
            Err(1)
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
    fn trait_message_metadata_return_value_is_result() {
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

    #[test]
    fn fallible_constructor_metadata_is_nested_result() {
        let metadata = generate_metadata();
        let constructor = metadata.spec().constructors().iter().next().unwrap();

        assert_eq!("try_new", constructor.label());
        let type_spec = constructor.return_type().opt_type().unwrap();
        assert_eq!(
            "ink_primitives::ConstructorResult",
            format!("{}", type_spec.display_name())
        );
        let ty = metadata.registry().resolve(type_spec.ty().id()).unwrap();

        assert_eq!("Result", format!("{}", ty.path()));
        match ty.type_def() {
            scale_info::TypeDef::Variant(variant) => {
                assert_eq!(2, variant.variants().len());

                // Outer Result
                let outer_ok_variant = &variant.variants()[0];
                let outer_ok_field = &outer_ok_variant.fields()[0];
                let outer_ok_ty = metadata
                    .registry()
                    .resolve(outer_ok_field.ty().id())
                    .unwrap();
                assert_eq!("Ok", outer_ok_variant.name());

                // Inner Result
                let inner_ok_ty = match outer_ok_ty.type_def() {
                    scale_info::TypeDef::Variant(variant) => {
                        assert_eq!(2, variant.variants().len());

                        let inner_ok_variant = &variant.variants()[0];
                        assert_eq!("Ok", inner_ok_variant.name());

                        let inner_ok_field = &inner_ok_variant.fields()[0];
                        metadata
                            .registry()
                            .resolve(inner_ok_field.ty().id())
                            .unwrap()
                    }
                    td => panic!("Expected a Variant type def enum, got {:?}", td),
                };

                let unit_ty = scale_info::TypeDef::Tuple(
                    scale_info::TypeDefTuple::new_portable(vec![]),
                );

                assert_eq!(
                    &unit_ty,
                    inner_ok_ty.type_def(),
                    "Ok variant should be a unit `()` type"
                );

                let err_variant = &variant.variants()[1];
                let err_field = &err_variant.fields()[0];
                let err_ty_result = metadata.registry().resolve(err_field.ty().id());
                assert_eq!("Err", err_variant.name());
                assert!(
                    err_ty_result.is_some(),
                    "Error variant must be encoded with SCALE"
                );
            }
            td => panic!("Expected a Variant type def enum, got {:?}", td),
        }
    }
}
