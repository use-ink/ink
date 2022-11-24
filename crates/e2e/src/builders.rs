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

use ink_env::{
    call::{
        utils::{
            Set,
            Unset,
        },
        ExecutionInput,
    },
    Environment,
};
use scale::Encode;

/// The type returned from `ContractRef` constructors, partially initialized with the execution
/// input arguments.
type CreateBuilderPartial<E, Args, R> = ink_env::call::CreateBuilder<
    E,
    Unset<<E as Environment>::Hash>,
    Unset<u64>,
    Unset<<E as Environment>::Balance>,
    Set<ExecutionInput<Args>>,
    Unset<ink_env::call::state::Salt>,
    R,
>;

// /// Fully initialized builder, allowing access to the
// type CreateBuilderReady<E: Environment, Args, R> = ink_env::call::CreateBuilder<
//     E,
//     Set<E::Hash>,
//     Set<u64>,
//     Set<E::Balance>,
//     Set<ExecutionInput<Args>>,
//     Set<ink_env::call::state::Salt>,
//     R,
// >;

/// Shim onto the `CreateBuilder` to allow access to the `ExecutionInput` args.
pub struct ConstructorBuilder<E: Environment, Args: Encode, R> {
    inner: CreateBuilderPartial<E, Args, R>,
}

impl<E: Environment, Args: Encode, R> From<CreateBuilderPartial<E, Args, R>>
    for ConstructorBuilder<E, Args, R>
{
    fn from(inner: CreateBuilderPartial<E, Args, R>) -> Self {
        Self { inner }
    }
}

impl<E: Environment, Args: Encode, R> ConstructorBuilder<E, Args, R> {
    /// Returns encoded constructor args.
    pub fn exec_input(self) -> Vec<u8> {
        // set all the other properties to default values, we only require the `exec_input`.
        self.inner
            .endowment(0u32.into())
            .code_hash(ink_primitives::Clear::clear())
            .salt_bytes(Vec::new())
            .params()
            .exec_input()
            .encode()
    }
}
