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

#[cfg(not(feature = "std"))]
extern crate alloc;

mod access_env;
mod contract;
mod dispatcher;
mod error;
mod msg;
mod storage;

pub use ink_lang2_macro::contract;

pub use self::{
    access_env::{
        AccessEnv,
        AccessEnvMut,
    },
    contract::{
        DispatchUsingMode,
        DispatchMode,
        StoragePair,
        Contract,
        ContractBuilder,
    },
    dispatcher::{
        Dispatch,
        DispatchList,
        DispatchableFn,
        DispatchableFnMut,
        Dispatcher,
        DispatcherMut,
        EmptyDispatchList,
        PushDispatcher,
        UnreachableDispatcher,
    },
    error::{
        DispatchError,
        DispatchResult,
        DispatchRetCode,
    },
    msg::{
        FnInput,
        FnOutput,
        FnSelector,
        Message,
    },
    storage::Storage,
};
