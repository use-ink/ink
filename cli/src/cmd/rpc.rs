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

use crate::cmd::{Result, CommandError};
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

struct Rpc {
    state: StateClient<Hash>,
    chain: ChainClient<BlockNumber, Hash, Vec<u8>, Vec<u8>>,
    author: AuthorClient<Hash, Hash>,
}

impl Rpc {
    fn connect(url: &str) -> impl Future<Item=Rpc, Error=RpcError> {
        http::connect(url)
            .join3(http::connect(url), http::connect(url))
            .map(|(state, chain, author)| Rpc { state, chain, author })
    }

    fn fetch_nonce(&self, account: &AccountId) -> impl Future<Item=u64, Error=RpcError> {
        let account_nonce_key = <srml_system::AccountNonce<Runtime>>::key_for(account);
        let storage_key = blake2_256(&account_nonce_key).to_vec();

        self.state.storage(StorageKey(storage_key), None)
            .map(|data| {
                data.map_or(0, |d| {
                    Decode::decode(&mut &d.0[..]).expect("Account nonce is valid Index")
                })
            })
            .map_err(Into::into)
    }

    fn fetch_genesis_hash(&self) -> impl Future<Item=Option<H256>, Error=RpcError> {
        self.chain.block_hash(Some(NumberOrHex::Number(0)))
    }

    fn submit_extrinsic(&self, extrinsic: UncheckedExtrinsic) -> impl Future<Item=H256, Error=RpcError> {
        self.author.submit_extrinsic(extrinsic.encode().into())
    }

    fn submit(self, signer: Pair, call: Call) -> impl Future<Item=H256, Error=CommandError> {
        let account_nonce = self.fetch_nonce(&signer.public()).map_err(Into::into);
        let genesis_hash = self.fetch_genesis_hash()
            .map_err(Into::into)
            .and_then(|genesis_hash| {
                future::result(genesis_hash.ok_or("Genesis hash not found".into()))
            });

        account_nonce
            .join(genesis_hash)
            .and_then(move |(index, genesis_hash)| {
                let extrinsic = Self::create_extrinsic(index, call, genesis_hash, &signer);
                self.submit_extrinsic(extrinsic).map_err(Into::into)
            })
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
    }
}

pub fn submit(url: &str, signer: Pair, call: Call) -> Result<Hash> {
    let submit = Rpc::connect(url)
        .map_err(Into::into)
        .and_then(|rpc| rpc.submit(signer, call));

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(submit)
}
