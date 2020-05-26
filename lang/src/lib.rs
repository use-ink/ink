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

#[cfg(feature = "ink-generate-abi")]
mod abi;

mod contract2;
mod cross_calling;
mod dispatcher2;
mod env_access;
mod error;
mod testable;
mod traits2;

/// Re-exports all entities from the new revision of this crate.
pub mod v2 {
    pub use super::{
        contract2::*,
        dispatcher2::*,
        traits2::*,
    };
}

pub use ink_lang_macro::contract;

#[cfg(feature = "ink-generate-abi")]
pub use self::abi::GenerateAbi;

pub use self::{
    contract2::{
        DispatchMode,
        DispatchUsingMode,
        Placeholder,
    },
    cross_calling::{
        ForwardCall,
        ForwardCallMut,
        ToAccountId,
    },
    env_access::{
        Env,
        EnvAccess,
        StaticEnv,
    },
    error::{
        DispatchError,
        DispatchResult,
        DispatchRetCode,
    },
    testable::InstantiateTestable,
};
