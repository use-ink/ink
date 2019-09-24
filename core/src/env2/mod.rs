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

//! Environment definitions and access.

pub mod call;
mod env_access;
mod error;
pub mod property;
mod srml;
mod test;
mod traits;
mod utils;

pub use self::{
    env_access::{
        EnvAccess,
    },
    error::{
        CallError,
        CreateError,
        Error,
        Result,
    },
    traits::{
        BuildCall,
        BuildCreate,
        BuildEvent,
        Env,
        EnvTypes,
        GetProperty,
        SetProperty,
    },
    utils::{
        EnlargeTo,
        Reset,
    },
};
