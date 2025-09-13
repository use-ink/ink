// Copyright (C) Use Ink (UK) Ltd.
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
use regex::Regex;
use std::{
    collections::BTreeMap,
    path::PathBuf,
};

/// Generates a unique salt based on the system time.
pub fn salt() -> Option<[u8; 32]> {
    use funty::Fundamental as _;

    let mut arr = [0u8; 32];
    let t: [u8; 16] = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|err| panic!("unable to get unix time: {err}"))
        .as_millis()
        .as_u128()
        .to_le_bytes();
    arr[..16].copy_from_slice(t.as_slice());
    arr[16..].copy_from_slice(t.as_slice());
    Some(arr)
}

/// Registry of contracts that can be loaded.
pub struct ContractsRegistry {
    contracts: BTreeMap<String, PathBuf>,
}

impl ContractsRegistry {
    /// Create a new registry with the given contracts.
    pub fn new<P: Into<PathBuf>>(contracts: impl IntoIterator<Item = P>) -> Self {
        let contracts = contracts
            .into_iter()
            .map(|path| {
                let contract_binary_path: PathBuf = path.into();
                let contract_name =
                    contract_binary_path.file_stem().unwrap_or_else(|| {
                        panic!(
                            "Invalid contract binary path `{}`",
                            contract_binary_path.display(),
                        )
                    });
                (
                    contract_name.to_string_lossy().to_string(),
                    contract_binary_path,
                )
            })
            .collect();

        Self { contracts }
    }

    /// Load the binary code for the given contract.
    pub fn load_code(&self, contract: &str) -> Vec<u8> {
        let contract_binary_path = self
            .contracts
            .iter().find_map(|(name, path)| {
                let re = Regex::new(r"-features-.+$").expect("failed creating regex");
                let key = re.replace_all(name, "");
                if key == contract || key.replace('_', "-") == contract {
                    return Some(path);
                }
                None
            })
            .unwrap_or_else(||
                panic!(
                    "Unknown contract {contract}. Available contracts: {:?}.\n\
                     For a contract to be built, add it as a dependency to the `Cargo.toml`",
                    self.contracts.keys()
                )
            );
        let code = std::fs::read(contract_binary_path).unwrap_or_else(|err| {
            panic!(
                "Error loading '{}': {:?}",
                contract_binary_path.display(),
                err
            )
        });
        log_info(&format!("{:?} has {} KiB", contract, code.len() / 1024));
        code
    }
}

/// Returns the `H256` hash of the code slice.
pub fn code_hash(code: &[u8]) -> [u8; 32] {
    h256_hash(code)
}

/// Returns the `H256` hash of the given `code` slice.
fn h256_hash(code: &[u8]) -> [u8; 32] {
    use sha3::{
        Digest,
        Keccak256,
    };
    let hash = Keccak256::digest(code);
    let sl = hash.as_slice();
    assert!(sl.len() == 32, "expected length of 32");
    let mut arr = [0u8; 32];
    arr.copy_from_slice(sl);
    arr
}
