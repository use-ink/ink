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

//! Definitions and facilities mainly used by the ink! codegen.

mod dispatch;
mod env;
mod event;
mod implies_return;
mod trait_def;
pub mod utils;

pub use self::{
    dispatch::{
        deny_payment,
        execute_constructor,
        finalize_message,
        initialize_contract,
        initiate_message,
        ContractCallBuilder,
        ContractRootKey,
        DispatchInput,
        DispatchOutput,
        ExecuteConstructorConfig,
        ExecuteMessageConfig,
    },
    env::{
        Env,
        StaticEnv,
    },
    event::{
        EmitEvent,
        EventLenTopics,
        EventRespectsTopicLimit,
        EventTopics,
        RespectTopicLimit,
    },
    implies_return::ImpliesReturn,
    trait_def::{
        TraitCallBuilder,
        TraitCallForwarder,
        TraitMessagePayable,
        TraitMessageSelector,
    },
};
