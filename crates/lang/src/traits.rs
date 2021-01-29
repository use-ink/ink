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

use core::marker::PhantomData;
use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
        },
        CallBuilder,
        ExecutionInput,
        Selector,
    },
    Environment,
};
use ink_storage::traits::SpreadLayout;

/// Trait used to indicate that an ink! trait definition has been checked
/// by the `#[ink::trait_definition]` proc. macro.
#[doc(hidden)]
pub unsafe trait CheckedInkTrait<T> {}

/// Trait used by `#[ink::trait_definition]` to ensure that the associated
/// return type for each trait message is correct.
#[doc(hidden)]
pub trait ImpliesReturn<T> {}

impl<T> ImpliesReturn<T> for T {}
impl<T, E, Callee, GasCost, TransferredValue, Args> ImpliesReturn<T>
    for CallBuilder<
        E,
        Callee,
        GasCost,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<T>>,
    >
where
    E: Environment,
{
}

impl<E, Callee, GasCost, TransferredValue, Args> ImpliesReturn<()>
    for CallBuilder<
        E,
        Callee,
        GasCost,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<()>,
    >
where
    E: Environment,
{
}

/// Dispatchable functions that have inputs.
#[doc(hidden)]
pub trait FnInput {
    /// The tuple-type of all inputs.
    type Input: scale::Decode + 'static;
}

/// Dispatchable functions that have an output.
#[doc(hidden)]
pub trait FnOutput {
    /// The output type.
    type Output: scale::Encode + 'static;
}

/// The selector of dispatchable functions.
#[doc(hidden)]
pub trait FnSelector {
    /// The selector.
    const SELECTOR: Selector;
}

/// The storage state that the dispatchable function acts on.
#[doc(hidden)]
pub trait FnState {
    /// The storage state.
    type State: SpreadLayout + Sized;
}

/// A dispatchable contract constructor message.
#[doc(hidden)]
pub trait Constructor: FnInput + FnSelector + FnState {
    const CALLABLE: fn(<Self as FnInput>::Input) -> <Self as FnState>::State;
}

/// A `&self` dispatchable contract message.
#[doc(hidden)]
pub trait MessageRef: FnInput + FnOutput + FnSelector + FnState {
    const CALLABLE: fn(
        &<Self as FnState>::State,
        <Self as FnInput>::Input,
    ) -> <Self as FnOutput>::Output;
}

/// A `&mut self` dispatchable contract message.
#[doc(hidden)]
pub trait MessageMut: FnInput + FnOutput + FnSelector + FnState {
    const CALLABLE: fn(
        &mut <Self as FnState>::State,
        <Self as FnInput>::Input,
    ) -> <Self as FnOutput>::Output;
}

/// Indicates that some compile time expression is expected to be `true`.
#[doc(hidden)]
pub trait True {}

/// This type is known to ink! to implement all defined ink! trait definitions.
/// This property can be guaranteed by `#[ink::trait_definition]` proc. macro.
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
/// <::ink_lang::TraitDefinitionRegistry as MyTrait>::__ink_DynamicCallForwarder
/// ```
/// Normal implementations of ink! trait definitions default the new
/// `__ink_DynamicCallForwarder` associated type to `::ink_lang::NoDynamicCallForwarder`.
///
/// This is the technique used by ink! to resolve `&dyn MyTrait`, `&mut dyn MyTrait`
/// in message parameters or `dyn MyTrait` in ink! storage fields to concrete types
/// that ink! can serialize and deserialize as if it was an `AccountId` and call
/// ink! messages on it according to the ink! trait definition interface.
#[doc(hidden)]
pub struct ConcreteImplementers<E> {
    marker: PhantomData<fn() -> E>,
}

unsafe impl<E, const N: usize> CheckedInkTrait<[(); N]> for ConcreteImplementers<E> {}

/// The default type that ink! trait definition implementations use for the
/// `__ink_DynamicCallForwarder` associated type.
///
/// Read more about its use [here][TraitDefinitionRegistry].
#[doc(hidden)]
pub enum NoConcreteImplementer {}
