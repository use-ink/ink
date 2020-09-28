// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use ink_core::{
    env::{
        call::{
            utils::{
                ReturnType,
                Set,
            },
            CallBuilder,
            ExecutionInput,
            Selector,
        },
        EnvTypes,
    },
    storage2::traits::SpreadLayout,
};

/// Trait used to indicate that an ink! trait definition has been checked
/// by the `#[ink::trait_definition]` proc. macro.
pub unsafe trait CheckedInkTrait<T> {}

/// Trait used by `#[ink::trait_definition]` to ensure that the associated
/// return type for each trait message is correct.
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
    E: EnvTypes,
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
    E: EnvTypes,
{
}

/// Dispatchable functions that have inputs.
pub trait FnInput {
    /// The tuple-type of all inputs.
    type Input: scale::Decode + 'static;
}

/// Dispatchable functions that have an output.
pub trait FnOutput {
    /// The output type.
    type Output: scale::Encode + 'static;
}

/// The selector of dispatchable functions.
pub trait FnSelector {
    /// The selector.
    const SELECTOR: Selector;
}

/// The storage state that the dispatchable function acts on.
pub trait FnState {
    /// The storage state.
    type State: SpreadLayout + Sized;
}

/// A dispatchable contract constructor message.
pub trait Constructor: FnInput + FnSelector + FnState {
    const CALLABLE: fn(<Self as FnInput>::Input) -> <Self as FnState>::State;
}

/// A `&self` dispatchable contract message.
pub trait MessageRef: FnInput + FnOutput + FnSelector + FnState {
    const CALLABLE: fn(
        &<Self as FnState>::State,
        <Self as FnInput>::Input,
    ) -> <Self as FnOutput>::Output;
}

/// A `&mut self` dispatchable contract message.
pub trait MessageMut: FnInput + FnOutput + FnSelector + FnState {
    const CALLABLE: fn(
        &mut <Self as FnState>::State,
        <Self as FnInput>::Input,
    ) -> <Self as FnOutput>::Output;
}
