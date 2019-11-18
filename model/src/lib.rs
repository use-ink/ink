// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn)]
#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
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
