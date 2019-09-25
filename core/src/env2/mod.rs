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
mod dyn_env;
mod env_access;
mod error;
pub mod property;
mod srml;
mod test;
mod traits;
pub mod utils;

/// Error definitions specific to environment accesses.
pub mod errors {
    pub use super::error::{
        CallError,
        CreateError,
    };
}

pub use self::{
    dyn_env::{
        DynEnv,
        DynEnvAccess,
        DynEnvAccessMut,
    },
    env_access::{
        EnvAccess,
        EnvAccessMut,
    },
    error::{
        Error,
        Result,
    },
    traits::{
        CallParams,
        CreateParams,
        EmitEventParams,
        Env,
        EnvTypes,
        GetProperty,
        SetProperty,
    },
};
