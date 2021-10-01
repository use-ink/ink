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

use crate::codegen::TraitImplementedById;
use core::marker::PhantomData;

/// This type is known to ink! to implement all defined ink! trait definitions.
/// This property can be guaranteed by `#[ink::trait_definition]` procedural macro.
///
/// By the introduction of an new internal and hidden associated type called
/// `__ink_DynamicCallForwarder` for all ink! trait definitions it is possible
/// for ink! to map from any given ink! trait definition back to a concrete
/// Rust type.
/// Whenever the `ChainExtensionRegistry` implements an ink! trait definition
/// all calls are defaulted to produce linker errors (ideally compiler errors
/// if that was possible) and the only relevant implementation is the new
/// `__ink_DynamicCallForwarder` associated type that links to a concrete
/// type implementing `FromAccountId` and the ink! trait definition with
/// proper implementations.
///
/// Then ink! can map from the ink! trait definition `MyTrait` to this concrete
/// dynamic call forwarder type by:
/// ```no_compile
/// <::ink_lang::reflect::TraitDefinitionRegistry as MyTrait>::__ink_DynamicCallForwarder
/// ```
/// Normal implementations of ink! trait definitions default the new
/// `__ink_DynamicCallForwarder` associated type to `::ink_lang::NoDynamicCallForwarder`.
///
/// This is the technique used by ink! to resolve `&dyn MyTrait`, `&mut dyn MyTrait`
/// in message parameters or `dyn MyTrait` in ink! storage fields to concrete types
/// that ink! can serialize and deserialize as if it was an `AccountId` and call
/// ink! messages on it according to the ink! trait definition interface.
pub struct TraitDefinitionRegistry<E> {
    marker: PhantomData<fn() -> E>,
}

impl<E> crate::ContractEnv for TraitDefinitionRegistry<E>
where
    E: ink_env::Environment,
{
    type Env = E;
}

unsafe impl<E, const N: u32> TraitImplementedById<N> for TraitDefinitionRegistry<E> {}
