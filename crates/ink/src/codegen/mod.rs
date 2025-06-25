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

//! Definitions and facilities mainly used by the ink! codegen.

mod dispatch;
mod env;
mod implies_return;
pub mod sol;
mod trait_def;
pub mod utils;

pub use self::{
    dispatch::{
        deny_payment,
        ContractCallBuilder,
        DispatchInput,
        DispatchInputSol,
        DispatchOutput,
        DispatchOutputSol,
    },
    env::{
        Env,
        StaticEnv,
    },
    implies_return::ImpliesReturn,
    trait_def::{
        TraitCallBuilder,
        TraitCallForwarder,
        TraitCallForwarderFor,
        TraitMessageBuilder,
        TraitMessagePayable,
        TraitMessageSelector,
    },
};
