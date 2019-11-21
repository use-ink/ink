// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

mod builder;
mod create;
mod utils;

pub mod state {
    pub use crate::env2::call::{
        create::state::{
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
    create::{
        CreateBuilder,
        CreateParams,
        FromAccountId,
    },
    utils::{
        CallData,
        Selector,
    },
};
