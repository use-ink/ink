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
        utils::{ReturnType, Set, Unset},
        CreateBuilder, CreateParams, ExecutionInput,
    },
    Environment,
};
use scale::Encode;

/// The type returned from `ContractRef` constructors, partially initialized with the execution
/// input arguments.
pub type CreateBuilderPartial<E, ContractRef, Args, R> = CreateBuilder<
    E,
    ContractRef,
    Unset<<E as Environment>::Hash>,
    Unset<u64>,
    Unset<<E as Environment>::Balance>,
    Set<ExecutionInput<Args>>,
    Unset<ink_env::call::state::Salt>,
    Set<ReturnType<R>>,
>;

pub fn finalise_constructor<E, ContractRef, Args: Encode, R>(
    builder: CreateBuilderPartial<E, ContractRef, Args, R>,
) -> CreateParams<E, ContractRef, Args, [u8;0], R>
where
    E: Environment,
{
    builder
        .endowment(0u32.into())
        .code_hash(ink_primitives::Clear::CLEAR_HASH)
        .salt_bytes([])
        .params()
}
