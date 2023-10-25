// Copyright (C) Parity Technologies (UK) Ltd.
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

use crate::log_info;
use std::{
    collections::BTreeMap,
    path::PathBuf,
};

/// Generate a unique salt based on the system time.
pub fn salt() -> Vec<u8> {
    use funty::Fundamental as _;

    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("unable to get unix time: {err}"))
        .as_millis()
        .as_u128()
        .to_le_bytes()
        .to_vec()
}

/// A registry of contracts that can be loaded.
pub struct ContractsRegistry {
    contracts: BTreeMap<String, PathBuf>,
}

impl ContractsRegistry {
    /// Create a new registry with the given contracts.
    pub fn new<P: Into<PathBuf>>(contracts: impl IntoIterator<Item = P>) -> Self {
        let contracts = contracts
            .into_iter()
            .map(|path| {
                let wasm_path: PathBuf = path.into();
                let contract_name = wasm_path.file_stem().unwrap_or_else(|| {
                    panic!("Invalid contract wasm path '{}'", wasm_path.display(),)
                });
                (contract_name.to_string_lossy().to_string(), wasm_path)
            })
            .collect();

        Self { contracts }
    }

    /// Load the Wasm code for the given contract.
    pub fn load_code(&self, contract: &str) -> Vec<u8> {
        let wasm_path = self
            .contracts
            .get(&contract.replace('-', "_"))
            .unwrap_or_else(||
                panic!(
                    "Unknown contract {contract}. Available contracts: {:?}.\n\
                     For a contract to be built, add it as a dependency to the `Cargo.toml`, or add \
                     the manifest path to `#[ink_e2e::test(additional_contracts = ..)]`",
                    self.contracts.keys()
                )
            );
        let code = std::fs::read(wasm_path).unwrap_or_else(|err| {
            panic!("Error loading '{}': {:?}", wasm_path.display(), err)
        });
        log_info(&format!("{:?} has {} KiB", contract, code.len() / 1024));
        code
    }
}
