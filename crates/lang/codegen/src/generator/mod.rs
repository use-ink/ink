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

mod blake2b;
mod chain_extension;
mod contract;
mod cross_calling;
mod dispatch;
mod env;
mod events;
mod ink_test;
mod item_impls;
mod metadata;
mod selector;
mod storage;
mod trait_def;

pub use self::{
    blake2b::Blake2x256,
    chain_extension::ChainExtension,
    contract::Contract,
    cross_calling::{
        CrossCalling,
        CrossCallingConflictCfg,
    },
    dispatch::Dispatch,
    env::Env,
    events::Events,
    ink_test::InkTest,
    item_impls::ItemImpls,
    metadata::Metadata,
    selector::{
        SelectorBytes,
        SelectorId,
    },
    storage::Storage,
    trait_def::TraitDefinition,
};
