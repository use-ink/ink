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

use crate::cmd::Result;
use node_runtime::{
    Call,
    Runtime,
    UncheckedExtrinsic,
};
use srml_contracts::{
    Call as ContractsCall,
    Trait,
};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
};

type CargoToml = HashMap<String, toml::Value>;

type Gas = <Runtime as Trait>::Gas;

fn get_contract_wasm_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml_path = manifest_dir.join("Cargo.toml");

    let mut content = String::new();
    let mut file = fs::File::open(cargo_toml_path)?;
    file.read_to_string(&mut content)?;

    let cargo_toml: CargoToml = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let contract_name = cargo_toml
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or("Failed to find valid name property in Cargo.toml")?;

    Ok(manifest_dir
        .join("target")
        .join(format!("{}-pruned.wasm", contract_name)))
}

fn load_contract_code(path: Option<PathBuf>) -> Result<Vec<u8>> {
    let contract_wasm_path = path.map(Ok).unwrap_or_else(get_contract_wasm_path)?;

    let mut data = Vec::new();
    let mut file = fs::File::open(contract_wasm_path)?;
    file.read_to_end(&mut data)?;

    return Ok(data)
}

fn create_put_code_extrinsic(gas: Gas, code: Vec<u8>) -> UncheckedExtrinsic {
    let call = Call::Contracts(ContractsCall::put_code(gas, code));
    UncheckedExtrinsic::new_unsigned(call)
}

fn sign_and_submit(_extrinsic: &mut UncheckedExtrinsic) {
    //    extrinsic.signature =
    unimplemented!()
}

pub(crate) fn execute_deploy(
    _on_dev: bool,
    gas: u64,
    contract_wasm_path: Option<PathBuf>,
) -> Result<()> {
    let code = load_contract_code(contract_wasm_path)?;
    let _extrinsic = create_put_code_extrinsic(gas, code);

    // 3. sign extrinsic
    // 4. Submit extrinsic via RPC
    // 5. Display code hash as result
    Ok(())
}
