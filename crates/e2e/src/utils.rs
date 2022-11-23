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
pub fn extract_wasm(contract: &contract_metadata::ContractMetadata) -> Vec<u8> {
    let code = contract
        .source
        .wasm
        .clone()
        .expect("contract bundle is missing `source.wasm`");
    log_info(&format!(
        "{:?} has {} KiB",
        contract.contract.name,
        code.0.len() / 1024
    ));
    code.0
}

/// Converts a `H256` runtime hash to the hash type of the
/// given ink! environment.
pub fn runtime_hash_to_ink_hash<'a, E>(
    runtime_hash: &'a super::H256,
) -> <E as ink_env::Environment>::Hash
where
    E: ink_env::Environment,
    <E as ink_env::Environment>::Hash: TryFrom<&'a [u8]>,
{
    let runtime_hash_slice: &[u8] = runtime_hash.as_ref();
    TryFrom::try_from(runtime_hash_slice).unwrap_or_else(|_|
        panic!("unable to convert hash slice from runtime into default ink! environment hash type")
    )
}
