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

use std::{
    io::Error as IoError,
    result::Result as StdResult,
};

use jsonrpc_core_client::RpcError;
use substrate_primitives::crypto::SecretStringError;
use subxt::Error as SubXtError;
use zip::result::ZipError;

/// An error that can be encountered while executing commands.
#[derive(Debug, derive_more::From, derive_more::Display)]
pub enum CommandError {
    Io(IoError),
    #[display(fmt = "Command unimplemented")]
    UnimplementedCommand,
    #[display(fmt = "Abstraction layer unimplemented")]
    UnimplementedAbstractionLayer,
    Rpc(RpcError),
    #[display(fmt = "Secret string error")]
    SecretString(SecretStringError),
    SubXt(SubXtError),
    ZipError(ZipError),
    BuildFailed,
    #[display(fmt = "Error invoking `cargo metadata`")]
    CargoMetadata(cargo_metadata::Error),
    WasmDeserialization(parity_wasm::elements::Error),
    #[display(fmt = "Optimizer failed")]
    Optimizer(pwasm_utils::OptimizerError),
    Other(String),
}

impl From<&str> for CommandError {
    fn from(error: &str) -> Self {
        CommandError::Other(error.into())
    }
}

/// Result type that has a `CommandError`.
pub type Result<T> = StdResult<T, CommandError>;
