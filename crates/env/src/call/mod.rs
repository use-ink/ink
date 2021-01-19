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

//! Utilities to call or instantiate contracts on the chain.

mod call_builder;
mod common;
mod create_builder;
mod execution_input;
mod selector;

/// Utility types for the cross-contract calling API.
pub mod utils {
    pub use super::{
        call_builder::IndicateReturnType,
        common::{
            ReturnType,
            Set,
            Unset,
            Unwrap,
        },
        execution_input::{
            ArgsList,
            Argument,
            ArgumentList,
            ArgumentListEnd,
            EmptyArgumentList,
        },
    };
}

pub use self::{
    call_builder::{
        build_call,
        CallBuilder,
        CallParams,
    },
    create_builder::{
        build_create,
        state,
        CreateBuilder,
        CreateParams,
        FromAccountId,
    },
    execution_input::ExecutionInput,
    selector::Selector,
};
