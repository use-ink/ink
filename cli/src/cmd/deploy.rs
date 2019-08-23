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
    Event,
};

use srml_contracts::{
    RawEvent as ContractsEvent,
};

use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
};
use runtime_primitives::generic::Era;
use substrate_primitives::{H256, crypto::Pair, sr25519};
use subxt::{srml::{balances::Balances, contracts::{Contracts, ContractsXt}, system::System}};
use futures::future::Future;

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
    extrinsic_result
        .events
        .iter()
        .find_map(|event| {
            if let Event::contracts(ContractsEvent::CodeStored(hash)) = event {
                Some(hash.clone())
            } else {
                None
            }
        })
        .ok_or("Failed to find contract.CodeStored event".into())
}

#[derive(Debug)]
struct Runtime;

impl System for Runtime {
    type Index = <node_runtime::Runtime as srml_system::Trait>::Index;
    type BlockNumber = <node_runtime::Runtime as srml_system::Trait>::BlockNumber;
    type Hash = <node_runtime::Runtime as srml_system::Trait>::Hash;
    type Hashing = <node_runtime::Runtime as srml_system::Trait>::Hashing;
    type AccountId = <node_runtime::Runtime as srml_system::Trait>::AccountId;
    type Lookup = <node_runtime::Runtime as srml_system::Trait>::Lookup;
    type Header = <node_runtime::Runtime as srml_system::Trait>::Header;
    type Event = <node_runtime::Runtime as srml_system::Trait>::Event;

    type SignedExtra = (
        srml_system::CheckVersion<node_runtime::Runtime>,
        srml_system::CheckGenesis<node_runtime::Runtime>,
        srml_system::CheckEra<node_runtime::Runtime>,
        srml_system::CheckNonce<node_runtime::Runtime>,
        srml_system::CheckWeight<node_runtime::Runtime>,
        srml_balances::TakeFees<node_runtime::Runtime>,
    );
    fn extra(nonce: Self::Index) -> Self::SignedExtra {
        (
            srml_system::CheckVersion::<node_runtime::Runtime>::new(),
            srml_system::CheckGenesis::<node_runtime::Runtime>::new(),
            srml_system::CheckEra::<node_runtime::Runtime>::from(Era::Immortal),
            srml_system::CheckNonce::<node_runtime::Runtime>::from(nonce),
            srml_system::CheckWeight::<node_runtime::Runtime>::new(),
            srml_balances::TakeFees::<node_runtime::Runtime>::from(0),
        )
    }
}

impl Balances for Runtime {
    type Balance = <node_runtime::Runtime as srml_balances::Trait>::Balance;
}

impl Contracts for Runtime {}

pub(crate) fn execute_deploy(
    url: url::Url,
    surl: &str,
    password: Option<&str>,
    gas: u64,
    contract_wasm_path: Option<PathBuf>,
) -> Result<()> {
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
    println!("Code hash: {:?}", code_hash);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Write,
        path,
    };

    #[test] #[ignore] // depends on a local substrate node running
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
        let result = super::execute_deploy(url, "//Alice", None, 500_000, Some(wasm_path));

        assert!(result.is_ok());
    }
}
