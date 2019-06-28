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
use futures::future::{
    self,
    Future,
    Either,
};
use parity_codec::{Encode, Decode, Compact};
use jsonrpc_core_client::{
    transports::http,
    RpcError,
};
use srml_contracts::{
    Call as ContractsCall,
};
use runtime_support::{StorageMap};
use runtime_primitives::generic::Era;
use substrate_primitives::{blake2_256, sr25519::Pair, Pair as _, storage::StorageKey, H256};
use substrate_rpc::{
    author::AuthorClient,
    chain::{
        number::NumberOrHex,
        ChainClient,
    },
    state::StateClient
};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
};

type CargoToml = HashMap<String, toml::Value>;

type AccountId = <Runtime as srml_system::Trait>::AccountId;
type BlockNumber = <Runtime as srml_system::Trait>::BlockNumber;
type Index = <Runtime as srml_system::Trait>::Index;
type Hash = <Runtime as srml_system::Trait>::Hash;

fn get_contract_wasm_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(".");
    let cargo_toml_path = manifest_dir.join("Cargo.toml");

    let mut content = String::new();
    let mut file = fs::File::open(&cargo_toml_path)
        .map_err(|e|format!("Failed to open {}: {}", cargo_toml_path.display(), e))?;
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
        .map_err(|e|format!("Failed to open {}: {}", contract_wasm_path.display(), e))?;
    file.read_to_end(&mut data)?;

    return Ok(data)
}

fn fetch_nonce(url: &str, account: &AccountId) -> impl Future<Item=u64, Error=RpcError> {
    let account_nonce_key = <srml_system::AccountNonce<Runtime>>::key_for(account);
    let storage_key = blake2_256(&account_nonce_key).to_vec();

    http::connect(url)
        .and_then(|cli: StateClient<Hash>| {
            cli.storage(StorageKey(storage_key), None)
        })
        .map(|data| {
            data.map_or(0, |d| {
                Decode::decode(&mut &d.0[..]).expect("Account nonce is valid Index")
            })
        })
        .map_err(Into::into)
}

fn fetch_genesis_hash(url: &str) -> impl Future<Item=Option<H256>, Error=RpcError> {
    http::connect(url)
        .and_then(|cli: ChainClient<BlockNumber, Hash, Vec<u8>, Vec<u8>>| cli.block_hash(Some(NumberOrHex::Number(0))))
}

fn create_extrinsic(index: Index, function: Call, block_hash: Hash, signer: &Pair) -> UncheckedExtrinsic {
    let era = Era::immortal();

    let raw_payload = (Compact(index), function, era, block_hash);
    let signature = raw_payload.using_encoded(|payload|
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            signer.sign(payload)
        }
    );

    UncheckedExtrinsic::new_signed(
        index,
        raw_payload.1,
        signer.public().into(),
        signature.into(),
        era,
    )

//    println!("0x{}", hex::encode(&extrinsic.encode()));
}

fn submit(url: &'static str, signer: Pair, call: Call) -> Result<Hash> {
    let genesis_hash = fetch_genesis_hash(url).map_err(Into::into);
    let account_nonce = fetch_nonce(url, &signer.public()).map_err(Into::into);

    let sign_and_submit = account_nonce
        .join(genesis_hash)
        .and_then(move |(index, genesis_hash)| {
            if let Some(block_hash) = genesis_hash {
                let extrinsic = create_extrinsic(index, call, block_hash, &signer);
                let submit = http::connect(url)
                    .and_then(move |cli: AuthorClient<Hash, Hash>| {
                        cli.submit_extrinsic(extrinsic.encode().into())
                    })
                    .map_err(Into::into);
                Either::A(submit)

            } else {
                Either::B(future::err("Genesis hash not found".into()))
            }
        });

    let mut rt = tokio::runtime::Runtime::new()?;
    let res = rt.block_on(sign_and_submit);
    println!("{:?}", res);
    res
}

pub(crate) fn execute_deploy(
    _on_dev: bool,
    gas: u64,
    contract_wasm_path: Option<PathBuf>,
) -> Result<()> {
    let url = "http://localhost:9933";
    // todo: [AJ] supply signer pair opt
    let signer = substrate_keyring::AccountKeyring::Alice.pair();

    let code = load_contract_code(contract_wasm_path)?;
    let call = Call::Contracts(ContractsCall::put_code(gas, code));

    submit(url, signer, call)?;

    // 5. Display code hash as result
    Ok(())
}
