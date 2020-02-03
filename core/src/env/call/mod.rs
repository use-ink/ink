// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

mod builder;
mod instantiate;
mod utils;

pub mod state {
    pub use crate::env::call::{
        instantiate::state::{
            CodeHashAssigned,
            CodeHashUnassigned,
        },
        utils::seal::{
            Sealed,
            Unsealed,
        },
    };
}

pub use self::{
    builder::{
        CallBuilder,
        CallParams,
        ReturnType,
    },
    instantiate::{
        FromAccountId,
        InstantiateBuilder,
        InstantiateParams,
    },
    utils::{
        CallData,
        Selector,
    },
};
