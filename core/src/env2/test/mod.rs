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

//! Test environment for off-chain testing and utilities.

mod accessor;
pub mod account;
mod instance;
pub mod record;
pub mod storage;
mod typed_encoded;
pub mod types;

use self::{
    account::{
        Account,
        AccountKind,
        AccountsDb,
        ContractAccount,
    },
    instance::TestEnvInstance,
    record::{
        CallContractRecord,
        CreateContractRecord,
        EmitEventRecord,
        InvokeRuntimeRecord,
        Record,
        RestoreContractRecord,
    },
    typed_encoded::{
        AlreadyInitialized,
        TypedEncoded,
    },
};

pub use self::accessor::TestEnv;
