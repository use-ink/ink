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

use futures::future::Future;
use runtime_primitives::{
    generic::{Era, Header},
    traits::{IdentityLookup, Verify, BlakeTwo256},
    AnySignature,
};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
};
use substrate_primitives::{
    crypto::Pair,
    sr25519,
    H256,
};
use subxt::{
    balances::Balances,
    contracts::{
        Contracts,
        ContractsXt,
    },
    system::System,
};

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

fn extract_code_hash(extrinsic_result: subxt::ExtrinsicSuccess<Runtime>) -> Result<H256> {
    match extrinsic_result.find_event::<H256>("Contracts", "CodeStored") {
        Some(Ok(hash)) => Ok(hash),
        Some(Err(err)) => Err(format!("Failed to decode code hash: {}", err).into()),
        None => Err("Failed to find Contracts::CodeStored Event".into()),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Runtime;

impl System for Runtime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = substrate_primitives::H256;
    type Hashing = BlakeTwo256;
    type AccountId = <AnySignature as Verify>::Signer;
    type Address = srml_indices::address::Address<Self::AccountId, u32>;
    type Header = Header<Self::BlockNumber, BlakeTwo256>;
}

impl Balances for Runtime {
    type Balance = u64;
}

impl Contracts for Runtime {}

pub(crate) fn execute_deploy(
    url: url::Url,
    surl: &str,
    password: Option<&str>,
    gas: u64,
    contract_wasm_path: Option<PathBuf>,
) -> Result<String> {
    let signer = sr25519::Pair::from_string(surl, password)?;

    let code = load_contract_code(contract_wasm_path)?;

    let fut = subxt::ClientBuilder::<Runtime>::new()
        .set_url(url)
        .build()
        .and_then(|cli| cli.xt(signer, None))
        .and_then(move |xt| {
            xt.contracts(|call| call.put_code(gas, code))
                .submit_and_watch()
        });

    let mut rt = tokio::runtime::Runtime::new()?;
    let extrinsic_success = rt.block_on(fut)?;

    log::debug!("Deploy success: {:?}", extrinsic_success);

    let code_hash = extract_code_hash(extrinsic_success)?;
    Ok(format!("Code hash: {:?}", code_hash))
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use std::{
        fs,
        io::Write,
        path,
    };

    #[test]
    #[ignore] // depends on a local substrate node running
    fn deploy_contract() {
        const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
        let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");

        let out_dir = path::Path::new(env!("OUT_DIR"));

        let target_dir = path::Path::new("./target");
        let _ = fs::create_dir(target_dir);

        let wasm_path = out_dir.join("flipper-pruned.wasm");
        let mut file = fs::File::create(&wasm_path).unwrap();
        let _ = file.write_all(&wasm);

        let url = url::Url::parse("ws://localhost:9944").unwrap();
        let result =
            super::execute_deploy(url, "//Alice", None, 500_000, Some(wasm_path));

        assert_matches!(result, Ok(_));
    }
}
