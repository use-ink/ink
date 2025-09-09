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

/// Implements `AsRef` for a code generator type.
///
/// Code generators always have a shared `contract` reference to the contract.
/// They need to implement this trait in order to use other code generators.
macro_rules! impl_as_ref_for_generator {
    ( $generator_name:ident ) => {
        impl ::core::convert::AsRef<ir::Contract> for $generator_name<'_> {
            fn as_ref(&self) -> &ir::Contract {
                self.contract
            }
        }
    };
}

#[macro_use]
mod macros;

mod arg_list;
mod as_dependency;
mod blake2b;
mod contract;
mod dispatch;
mod env;
mod event;
mod ink_test;
mod item_impls;
mod metadata;
mod selector;
mod sol;
mod storage;
mod storage_item;
mod trait_def;

#[cfg(any(ink_abi = "sol", ink_abi = "all"))]
pub use self::sol::metadata::SolidityMetadata;
pub use self::{
    arg_list::{
        generate_argument_list,
        generate_reference_to_trait_info,
        input_bindings,
        input_bindings_tuple,
        input_message_idents,
        input_types,
        input_types_tuple,
        output_ident,
    },
    as_dependency::ContractReference,
    blake2b::Blake2x256,
    contract::Contract,
    dispatch::Dispatch,
    env::Env,
    event::Event,
    ink_test::InkTest,
    item_impls::ItemImpls,
    metadata::{
        Metadata,
        generate_type_spec,
    },
    selector::{
        SelectorBytes,
        SelectorId,
    },
    storage::Storage,
    storage_item::StorageItem,
    trait_def::TraitDefinition,
};
