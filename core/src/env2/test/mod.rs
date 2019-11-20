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
