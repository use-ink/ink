// Copyright (C) Use Ink (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(unexpected_cfgs)]

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
}

#[cfg(test)]
mod tests {
    use scale_info::{
        Type,
        TypeDef,
        TypeDefPrimitive,
        TypeDefTuple,
        form::PortableForm,
    };

    fn generate_metadata() -> ink_metadata::InkProject {
        unsafe extern "Rust" {
            fn __ink_generate_metadata() -> ink_metadata::InkProject;
        }

        unsafe { __ink_generate_metadata() }
    }

    /// Extract the type defs of the `Ok` and `Error` variants of a `Result` type.
    ///
    /// Panics if the type def is not a valid result
    fn extract_result<'a>(
        metadata: &'a ink_metadata::InkProject,
        ty: &'a Type<PortableForm>,
    ) -> (&'a Type<PortableForm>, &'a Type<PortableForm>) {
        assert_eq!(
            "Result",
            format!("{}", ty.path),
            "Message return type should be a Result"
        );
        match &ty.type_def {
            TypeDef::Variant(variant) => {
                assert_eq!(2, variant.variants.len());
                let ok_variant = &variant.variants[0];
                let ok_field = &ok_variant.fields[0];
                let ok_ty = resolve_type(metadata, ok_field.ty.id);
                assert_eq!("Ok", ok_variant.name);

                let err_variant = &variant.variants[1];
                let err_field = &err_variant.fields[0];
                let err_ty = resolve_type(metadata, err_field.ty.id);
                assert_eq!("Err", err_variant.name);

                (ok_ty, err_ty)
            }
            td => panic!("Expected a Variant type def enum, got {td:?}"),
        }
    }

    /// Resolve a type with the given id from the type registry
    fn resolve_type(
        metadata: &ink_metadata::InkProject,
        type_id: u32,
    ) -> &Type<PortableForm> {
        metadata
            .registry()
            .resolve(type_id)
            .unwrap_or_else(|| panic!("No type found in registry with id {type_id}"))
    }

    #[test]
    fn trait_message_metadata_return_value_is_result() {
        let metadata = generate_metadata();

        let message = metadata.spec().messages().iter().next().unwrap();
        assert_eq!("TraitDefinition::get_value", message.label());

        let type_spec = message.return_type().ret_type();
        let ty = resolve_type(&metadata, type_spec.ty().id);
        let (ok_ty, _) = extract_result(&metadata, ty);

        assert_eq!(TypeDef::Primitive(TypeDefPrimitive::U32), ok_ty.type_def);
    }

    #[test]
    fn fallible_constructor_metadata_is_nested_result() {
        let metadata = generate_metadata();
        let constructor = metadata.spec().constructors().iter().next().unwrap();

        assert_eq!("try_new", constructor.label());
        let type_spec = constructor.return_type().ret_type();
        assert_eq!(
            "ink_primitives::ConstructorResult",
            format!("{}", type_spec.display_name())
        );

        let outer_result_ty = resolve_type(&metadata, type_spec.ty().id);
        let (outer_ok_ty, outer_err_ty) = extract_result(&metadata, outer_result_ty);
        let (inner_ok_ty, _) = extract_result(&metadata, outer_ok_ty);

        assert_eq!(
            format!("{}", outer_err_ty.path),
            "ink_primitives::LangError"
        );

        let unit_ty = TypeDef::Tuple(TypeDefTuple::new_portable(vec![]));
        assert_eq!(
            unit_ty, inner_ok_ty.type_def,
            "Ok variant should be a unit `()` type"
        );
    }
}
