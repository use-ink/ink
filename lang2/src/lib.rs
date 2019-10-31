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

#[cfg(feature = "ink-generate-abi")]
mod abi;

mod contract;
mod cross_calling;
mod dispatcher;
mod error;
mod testable;
mod traits;

pub use ink_lang2_macro::contract;

#[cfg(feature = "ink-generate-abi")]
pub use self::abi::GenerateAbi;

pub use self::{
    contract::{
        Contract,
        ContractBuilder,
        DispatchMode,
        DispatchUsingMode,
    },
    cross_calling::{
        ForwardCall,
        ForwardCallMut,
        ToAccountId,
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
    testable::InstantiateTestable,
    traits::{
        AccessEnv,
        FnInput,
        FnOutput,
        FnSelector,
        Message,
        Storage,
    },
};
