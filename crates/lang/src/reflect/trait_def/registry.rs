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

/// Type that is guaranteed by ink! to implement all ink! trait definitions.
///
/// This guarantee is used by ink! itself and can be used by ink! smart contract
/// authors to query static information about known ink! trait definitions.
///
/// # Codegen
///
/// - The `#[ink::trait_definition]` procedural macro generates an associated type
///   called `__ink_TraitInfo` for each ink! trait definition.
/// - Furthermore the ink! codegen implements the ink! trait definition for the
///   `TraitDefinitionRegistry` with stub implementations for all methods that
///   guarantee that they are never called.
/// - For every implemented ink! trait definition an ink! trait info object type
///   is generated that is linked to the global `TraitDefinitionRegistry` through
///   the aforementioned `__ink_TraitInfo` associated type.
/// - This trait info object type itself implements various traits each providing
///   useful static reflection information to the rest of the codegen about the ink!
///   trait definition.
///
/// # Usage
///
/// ```
/// # use ink_lang as ink;
/// # use ink_lang::reflect::TraitDefinitionRegistry;
/// use ink_env::DefaultEnvironment;
///
/// #[ink::trait_definition]
/// pub trait TraitDefinition {
///     #[ink(message)]
///     fn message(&self);
/// }
///
/// /// Access the generated ink! trait info object type like this:
/// type TraitInfo = <TraitDefinitionRegistry<DefaultEnvironment>
///     as TraitDefinition>::__ink_TraitInfo;
/// ```
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
