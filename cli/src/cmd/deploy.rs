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
    Runtime,
    UncheckedExtrinsic,
};
use futures::Future;
use parity_codec::{Encode, Decode};
use jsonrpc_core_client::{transports::http};
use srml_contracts::{
    Call as ContractsCall,
    Trait,
};
use runtime_support::{StorageMap};
use substrate_primitives::{blake2_256, sr25519::Pair, Pair as _, storage::StorageKey};
use substrate_rpc::state::StateClient;
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
};

type CargoToml = HashMap<String, toml::Value>;

type Gas = <Runtime as Trait>::Gas;
type Index = <Runtime as srml_system::Trait>::Index;
type Hash = <Runtime as srml_system::Trait>::Hash;

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

fn submit(url: &str, signer: &Pair, extrinsic: &mut UncheckedExtrinsic) -> Result<()> {
    let account_nonce_key = <srml_system::AccountNonce<Runtime>>::key_for(signer.public());
    let account_nonce_key = blake2_256(&account_nonce_key).to_vec();

    let submit = http::connect(url)
        .and_then(|cli: StateClient<Hash>| cli.storage(StorageKey(account_nonce_key), None))
        .map(|data| {
            data.map(|d| {
                let res: Index = Decode::decode(&mut &d.0[..])
                    .expect("Account nonce is valid Index");
                res
            })
        })
        .and_then(|index: Option<Index>| {
            futures::future::ok(())
        });

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(submit).map_err(Into::into)
}

pub(crate) fn execute_deploy(
    _on_dev: bool,
    gas: u64,
    contract_wasm_path: Option<PathBuf>,
) -> Result<()> {
    // todo: [AJ] supply signer pair opt
    let signer = substrate_keyring::AccountKeyring::Alice.pair();

    let code = load_contract_code(contract_wasm_path)?;
    let call = Call::Contracts(ContractsCall::put_code(gas, code)).encode();

    // todo [AJ] construct extrinsic and sign, then submit
//    let extrinsic = UncheckedExtrinsic::new_unsigned(
//        signer.public().into(),
//
//    );

    // 3. sign extrinsic
    // 4. Submit extrinsic via RPC
    // 5. Display code hash as result
    Ok(())
}
