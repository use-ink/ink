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
use futures::future::{
    self,
    Future,
};
use parity_codec::{Encode, Decode, Compact};
use jsonrpc_core_client::{
    transports::http,
    RpcError,
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


type AccountId = <Runtime as srml_system::Trait>::AccountId;
type BlockNumber = <Runtime as srml_system::Trait>::BlockNumber;
type Index = <Runtime as srml_system::Trait>::Index;
type Hash = <Runtime as srml_system::Trait>::Hash;

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

pub fn submit(url: &'static str, signer: Pair, call: Call) -> Result<Hash> {
    let account_nonce = fetch_nonce(url, &signer.public()).map_err(Into::into);
    let genesis_hash = fetch_genesis_hash(url)
        .map_err(Into::into)
        .and_then(|genesis_hash| {
            future::result(genesis_hash.ok_or("Genesis hash not found".into()))
        });

    let sign_and_submit = account_nonce
        .join(genesis_hash)
        .and_then(move |(index, genesis_hash)| {
            let extrinsic = create_extrinsic(index, call, genesis_hash, &signer);
            http::connect(url)
                .and_then(move |cli: AuthorClient<Hash, Hash>| {
                    cli.submit_extrinsic(extrinsic.encode().into())
                })
                .map_err(Into::into)
        });

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(sign_and_submit)
}
