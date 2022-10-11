// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use super::log_info;

/// Extracts the Wasm blob from a contract bundle.
pub fn extract_wasm(contract_path: &str) -> Vec<u8> {
    log_info(&format!("opening {:?}", contract_path));
    let reader = std::fs::File::open(&contract_path).unwrap_or_else(|err| {
        panic!("contract path cannot be opened: {:?}", err);
    });
    let contract: contract_metadata::ContractMetadata = serde_json::from_reader(reader)
        .unwrap_or_else(|err| {
            panic!("error reading metadata: {:?}", err);
        });
    let code = contract
        .source
        .wasm
        .expect("contract bundle is missing `source.wasm`");
    log_info(&format!(
        "{:?} has {} KiB",
        contract_path,
        code.0.len() / 1024
    ));
    code.0
}
