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

use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
        },
        CreateBuilder,
        ExecutionInput,
        LimitParamsV2,
    },
    Environment,
};
use ink_primitives::abi::{
    AbiEncodeWith,
    Ink,
};

use crate::H256;

/// The type returned from `ContractRef` constructors, partially initialized with the
/// execution input arguments.
pub type CreateBuilderPartial<E, ContractRef, Args, R> = CreateBuilder<
    E,
    ContractRef,
    Set<LimitParamsV2>,
    Set<ExecutionInput<Args, Ink>>,
    Set<ReturnType<R>>,
>;

/// Get the encoded constructor arguments from the partially initialized `CreateBuilder`
pub fn constructor_exec_input<E, ContractRef, Args: AbiEncodeWith<Ink>, R>(
    builder: CreateBuilderPartial<E, ContractRef, Args, R>,
) -> Vec<u8>
where
    E: Environment,
{
    // set all the other properties to default values, we only require the `exec_input`.
    builder
        .endowment(0u32.into())
        .code_hash(H256::zero())
        .salt_bytes(None)
        .params()
        .exec_input()
        .encode()
}
