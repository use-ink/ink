// Copyright 2019 Parity Technologies (UK) Ltd.
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

use crate::cmd::Result;
use node_runtime::{
    Call,
};

use srml_contracts::{
    Call as ContractsCall,
};

use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
};
use substrate_primitives::H256;
use super::rpc;

type CargoToml = HashMap<String, toml::Value>;

fn get_contract_wasm_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(".");
    let cargo_toml_path = manifest_dir.join("Cargo.toml");

    let mut content = String::new();
    let mut file = fs::File::open(&cargo_toml_path)
        .map_err(|e| format!("Failed to open {}: {}", cargo_toml_path.display(), e))?;
    file.read_to_string(&mut content)?;

    let cargo_toml: CargoToml = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let contract_name = cargo_toml
        .get("package")
        .and_then(|value| value.as_table())
        .and_then(|t| t.get("name"))
        .and_then(|v| v.as_str())
        .ok_or("Failed to find valid name property in Cargo.toml")?;

    Ok(manifest_dir
        .join("target")
        .join(format!("{}-pruned.wasm", contract_name)))
}

fn load_contract_code(path: Option<PathBuf>) -> Result<Vec<u8>> {
    let contract_wasm_path = path.map(Ok).unwrap_or_else(get_contract_wasm_path)?;

    let mut data = Vec::new();
    let mut file = fs::File::open(&contract_wasm_path)
        .map_err(|e| format!("Failed to open {}: {}", contract_wasm_path.display(), e))?;
    file.read_to_end(&mut data)?;

    return Ok(data)
}

fn extract_code_hash(extrinsic_result: rpc::ExtrinsicSuccess) -> Result<H256> {
    extrinsic_result.events
        .iter()
        .find_map(|_event| {
            // todo [AJ] find CodeStored event
//            if let Event::CodeStored(hash) = event {
//                Some(event.data)
//            } else {
//                None
//            }
            None
        })
        .ok_or("Failed to find contract.CodeStored event".into())
}

pub(crate) fn execute_deploy(
    _on_dev: bool,
    gas: u64,
    contract_wasm_path: Option<PathBuf>,
) -> Result<()> {
    // todo: [AJ] pass in these arguments
    let url = "http://localhost:9944";
    let signer = substrate_keyring::AccountKeyring::Alice.pair();

    let code = load_contract_code(contract_wasm_path)?;
    let call = Call::Contracts(ContractsCall::put_code(gas, code));

    let extrinsic_success = rpc::submit(url, signer, call)?;
    println!("{:?}", extrinsic_success.block);

    Ok(())
}
