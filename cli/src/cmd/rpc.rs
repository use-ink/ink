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
use log;
use parity_codec::{Encode, Decode, Compact};
use jsonrpc_core_client::{
    transports::ws,
    RpcError,
    TypedSubscriptionStream,
};

use runtime_support::{StorageMap};
use runtime_primitives::{generic::Era};
use serde::{self, Deserialize, de::Error as DeError};
use substrate_primitives::{
    blake2_256,
    sr25519::Pair,
    Pair as _,
    storage::{
        StorageChangeSet,
        StorageKey,
    },
    H256,
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

/// Simple blob to hold an extrinsic without committing to its format and ensure it is serialized
/// correctly.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
pub struct OpaqueExtrinsic(pub Vec<u8>);

impl std::fmt::Debug for OpaqueExtrinsic {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", substrate_primitives::hexdisplay::HexDisplay::from(&self.0))
    }
}

impl<'a> serde::Deserialize<'a> for OpaqueExtrinsic {
    fn deserialize<D>(de: D) -> std::result::Result<Self, D::Error> where D: serde::Deserializer<'a> {
        let r = substrate_primitives::bytes::deserialize(de)?;
        Decode::decode(&mut &r[..]).ok_or(DeError::custom("Invalid value passed into decode"))
    }
}

/// Copy of runtime_primitives::generic::Block with Deserialize implemented
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Deserialize)]
//#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
//#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct Block {
    /// The block header.
//    pub header: Header,
    /// The accompanying extrinsics.
    pub extrinsics: Vec<OpaqueExtrinsic>,
}

/// Copy of runtime_primitives::generic::SignedBlock with Deserialize implemented
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Deserialize)]
pub struct SignedBlock {
    /// Full block.
    pub block: Block,
}

struct Query {
    state: StateClient<Hash>,
    chain: ChainClient<BlockNumber, Hash, (), SignedBlock>,
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

    fn fetch_genesis_hash(&self) -> impl Future<Item=Option<H256>, Error=RpcError> {
        self.chain.block_hash(Some(NumberOrHex::Number(0)))
    }

    fn fetch_block(&self, block_hash: H256) -> impl Future<Item=Option<SignedBlock>, Error=RpcError> {
        self.chain.block(Some(block_hash))
    }

    fn subscribe_events(&self) -> impl Future<Item=TypedSubscriptionStream<StorageChangeSet<H256>>, Error=RpcError> {
        let events_key = b"System Events";
        let storage_key = blake2_256(events_key);
        log::info!("Events storage key {:?}", storage_key);

        self.state.subscribe_storage(Some(vec![StorageKey(storage_key.to_vec())]))
    }
}

pub struct ExtrinsicSuccess {
    pub block: Hash,
    pub extrinsic: Hash,
    pub events: Vec<Event>
}

struct Author {
    cli: AuthorClient<Hash, Hash>,
}

impl Author {
    fn connect_ws(url: &str) -> impl Future<Item=Author, Error=RpcError> {
        ws::connect(url).unwrap() // todo: [AJ] remove unwrap
            .map(|cli| Author { cli })
    }

    /// Submit extrinsic and return corresponding Event if successful
    fn submit(self, query: Query, signer: Pair, call: Call) -> impl Future<Item=ExtrinsicSuccess, Error=CommandError> {
        let account_nonce = query.fetch_nonce(&signer.public()).map_err(Into::into);
        let genesis_hash = query.fetch_genesis_hash()
            .map_err(Into::into)
            .and_then(|genesis_hash| {
                future::result(genesis_hash.ok_or("Genesis hash not found".into()))
            });
        let events = query.subscribe_events().map_err(Into::into);

        account_nonce
            .join3(genesis_hash, events)
            .and_then(move |(index, genesis_hash, events)| {
                log::info!("Creating Extrinsic with genesis hash '{:?}' and account nonce '{:?}'", genesis_hash, index);
                let extrinsic = Self::create_extrinsic(index, call, genesis_hash, &signer);
                let ext_hash = H256(extrinsic.using_encoded(|encoded| blake2_256(encoded)));
                log::info!("Submitted Extrinsic {:?}", ext_hash);

                self.submit_and_watch(extrinsic)
                    .and_then(move |bh| {
                        log::info!("Fetching block {:?}", bh);
                        query.fetch_block(bh).map(move |b| (bh, b)).map_err(Into::into)
                    })
                    .and_then(|(h, b)| b.ok_or(format!("Failed to find block '{:#x}'", h).into()).map(|b| (h, b)).into_future())
                    .and_then(move |(bh, sb)| {
                        log::info!("Found block {:?}, with {} extrinsics", bh, sb.block.extrinsics.len());
                        Self::extract_events(ext_hash, &sb, bh, events)
                    })
            })
    }

    /// Submit an extrinsic, waiting for it to be finalized.
    /// If successful, returns the block hash.
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

        let check_era = srml_system::CheckEra::from(Era::Immortal);
        let check_nonce = srml_system::CheckNonce::from(index);
        let check_weight = srml_system::CheckWeight::from();
        let take_fees = srml_balances::TakeFees::from(0);
        let extra = (check_era, check_nonce, check_weight, take_fees);

        UncheckedExtrinsic::new_signed(
            raw_payload.1,
            signer.public().into(),
            signature.into(),
            extra,
        )
    }

    fn extract_events(
        ext_hash: H256,
        sb: &SignedBlock,
        bh: H256,
        events: TypedSubscriptionStream<StorageChangeSet<H256>>,
    ) -> impl Future<Item=ExtrinsicSuccess, Error=CommandError> {
        let ext_index = sb.block.extrinsics
            .iter()
            .position(|ext| {
                let hash = H256(ext.using_encoded(|encoded| blake2_256(encoded)));
                hash == ext_hash
            })
            .ok_or(format!("Failed to find Extrinsic with hash {:?}", ext_hash).into())
            .into_future();

        let block_hash = bh.clone();
        let block_events = events
            .map(|event| {
                let records =
                    event.changes
                        .iter()
                        .filter_map(|(_key, data)| {
                            if let Some(data) = data {
                                let record: EventRecord = Decode::decode(&mut &data.0[..]).unwrap(); // todo: [AJ] remove unwrap
                                Some(record)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                log::info!("Block {:?}, Events {:?}", event.block, records);
                (event.block, records)
            })
            .filter(move |(event_block, _)| *event_block == block_hash)
            .into_future()
            .map_err(|(e,_)| e.into())
            .map(|(events, _)| events);

        block_events
            .join(ext_index)
            .map(move |(events, ext_index)| {
                let events = events
                    .iter()
                    .flat_map(|(_, events)| events)
                    .filter_map(|e| {
                        if let srml_system::Phase::ApplyExtrinsic(i) = e.phase {
                            if i as usize == ext_index {
                                Some(e.event.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                ExtrinsicSuccess { block: bh, extrinsic: ext_hash.into(), events }
            })
    }
}

pub fn submit(url: &str, signer: Pair, call: Call) -> Result<ExtrinsicSuccess> {
    let submit = Query::connect_ws(url)
        .join(Author::connect_ws(url))
        .map_err(Into::into)
        .and_then(|(query, author)| author.submit(query, signer, call));

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(submit)
}
