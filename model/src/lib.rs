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

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn)]
#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_qualifications,
    unused_results
)]

#[macro_use]
mod state;

#[macro_use]
mod msg;

mod contract;
mod exec_env;
mod msg_handler;

pub use crate::{
    contract::{
        Contract,
        ContractDecl,
        ContractInstance,
        DeployHandler,
        EmptyContractState,
        NoDeployArgs,
        TestableContract,
    },
    exec_env::{
        EnvHandler,
        ExecutionEnv,
    },
    msg::Message,
    msg_handler::{
        CallData,
        Error,
        HandleCall,
        MessageHandler,
        MessageHandlerMut,
        MessageHandlerSelector,
        RawMessageHandler,
        RawMessageHandlerMut,
        Result,
        UnreachableMessageHandler,
    },
    state::ContractState,
};
