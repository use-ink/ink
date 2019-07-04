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
use futures::{
    future::{
        self,
        Future,
        IntoFuture,
    },
    stream::Stream,
};
use parity_codec::{Encode, Decode, Compact};
use jsonrpc_core_client::{
    transports::ws,
    RpcError,
    TypedSubscriptionStream,
};

use runtime_support::{StorageMap};
use runtime_primitives::generic::Era;
use substrate_primitives::{
    blake2_256,
    sr25519::Pair,
    Pair as _,
    storage::{
        StorageChangeSet,
        StorageKey,
    },
    H256
};
use substrate_rpc::{
    author::AuthorClient,
    chain::{
        number::NumberOrHex,
        ChainClient,
    },
    state::StateClient
};
use transaction_pool::txpool::watcher::Status;

type AccountId = <Runtime as srml_system::Trait>::AccountId;
type BlockNumber = <Runtime as srml_system::Trait>::BlockNumber;
type Index = <Runtime as srml_system::Trait>::Index;
type Hash = <Runtime as srml_system::Trait>::Hash;
type Event = <Runtime as srml_system::Trait>::Event;
type EventRecord = srml_system::EventRecord<Event, Hash>;

struct Query {
    state: StateClient<Hash>,
    chain: ChainClient<BlockNumber, Hash, Vec<u8>, Vec<u8>>,
}

impl Query {
    fn connect_ws(url: &str) -> impl Future<Item=Query, Error=RpcError> {
        ws::connect(url).unwrap() // todo: [AJ] remove unwraps
            .join(ws::connect(url).unwrap())
            .map(|(state, chain)| Query { state, chain })
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

    fn subscribe_events(&self) -> impl Future<Item=TypedSubscriptionStream<StorageChangeSet<H256>>, Error=RpcError> {
        let events_key = b"Events";
        let storage_key = blake2_256(&events_key[..]).to_vec();

        self.state.subscribe_storage(Some(vec![StorageKey(storage_key)]))
    }

    fn fetch_genesis_hash(&self) -> impl Future<Item=Option<H256>, Error=RpcError> {
        self.chain.block_hash(Some(NumberOrHex::Number(0)))
    }
}

struct Author {
    cli: AuthorClient<Hash, Hash>,
}

impl Author {
    fn connect_ws(url: &str) -> impl Future<Item=Author, Error=RpcError> {
        ws::connect(url).unwrap() // todo: [AJ] remove unwrap
            .map(|cli| Author { cli })
    }

    fn submit(self, query: &Query, signer: Pair, call: Call) -> impl Future<Item=H256, Error=CommandError> {
        let account_nonce = query.fetch_nonce(&signer.public()).map_err(Into::into);
        let genesis_hash = query.fetch_genesis_hash()
            .map_err(Into::into)
            .and_then(|genesis_hash| {
                future::result(genesis_hash.ok_or("Genesis hash not found".into()))
            });

        account_nonce
            .join(genesis_hash)
            .and_then(move |(index, genesis_hash)| {
                let extrinsic = Self::create_extrinsic(index, call, genesis_hash, &signer);
                self.submit_and_watch(extrinsic)
            })
    }

    /// Submit an extrinsic, waiting for it to be finalized.
    /// If successful, returns the block hash
    fn submit_and_watch(self, extrinsic: UncheckedExtrinsic) -> impl Future<Item=H256, Error=CommandError> {
        self.cli.watch_extrinsic(extrinsic.encode().into())
            .map_err(Into::into)
            .and_then(|stream| {
                stream
                    .filter_map(|status| {
                        match status {
                            Status::Future | Status::Ready | Status::Broadcast(_) => None, // ignore in progress extrinsic for now
                            Status::Finalized(block_hash) => Some(Ok(block_hash)),
                            Status::Usurped(_) => Some(Err("Extrinsic Usurped".into())),
                            Status::Dropped => Some(Err("Extrinsic Dropped".into())),
                            Status::Invalid => Some(Err("Extrinsic Invalid".into())),
                        }
                    })
                    .into_future()
                    .map_err(|(e,_)| e.into())
                    .and_then(|(result, _)| {
                        result
                            .ok_or(CommandError::from("Stream terminated"))
                            .and_then(|r| r)
                            .into_future()
                    })
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
    let submit = Query::connect_ws(url)
        .join(Author::connect_ws(url))
        .map_err(Into::into)
        .and_then(|(query, author)| author.submit(&query, signer, call));
//        .and_then(|rpc| rpc.submit(signer, call));

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(submit)
}
