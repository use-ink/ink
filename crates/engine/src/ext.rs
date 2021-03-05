// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Provides the same interface as Substrate's FRAME `contract` module.
//!
//! See that module for more documentation.

use crate::{
    exec_context::ExecContext,
    storage::Storage,
    test_api::{
        EmittedEvent,
        REC_INSTANCE,
    },
};

use core::cell::RefCell;

type Result = core::result::Result<(), Error>;

pub struct EnvInstance {
    /// The environment storage.
    pub storage: Storage<Vec<u8>, Vec<u8>>,
    /// Current execution context.
    pub exec_context: Option<ExecContext>,
}

thread_local!(
    pub static ENV_INSTANCE: RefCell<EnvInstance> = RefCell::new(EnvInstance {
        storage: Storage::new(),
        exec_context: Some(ExecContext {
            caller: vec![0x01; 32].into(),
            callee: vec![0x01; 32].into(),
        }),
    });
);

macro_rules! define_error_codes {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident = $discr:literal,
        )*
    ) => {
        /// Every error that can be returned to a contract when it calls any of the host functions.
        #[repr(u32)]
        pub enum Error {
            $(
                $( #[$attr] )*
                $name = $discr,
            )*
            /// Returns if an unknown error was received from the host module.
            UnknownError,
        }

        impl From<ReturnCode> for Result {
            #[inline]
            fn from(return_code: ReturnCode) -> Self {
                match return_code.0 {
                    0 => Ok(()),
                    $(
                        $discr => Err(Error::$name),
                    )*
                    _ => Err(Error::UnknownError),
                }
            }
        }
    };
}
define_error_codes! {
    /// The called function trapped and has its state changes reverted.
    /// In this case no output buffer is returned.
    /// Can only be returned from `seal_call` and `seal_instantiate`.
    CalleeTrapped = 1,
    /// The called function ran to completion but decided to revert its state.
    /// An output buffer is returned when one was supplied.
    /// Can only be returned from `seal_call` and `seal_instantiate`.
    CalleeReverted = 2,
    /// The passed key does not exist in storage.
    KeyNotFound = 3,
    /// Transfer failed because it would have brought the sender's total balance
    /// below the subsistence threshold.
    BelowSubsistenceThreshold = 4,
    /// Transfer failed for other not further specified reason. Most probably
    /// reserved or locked balance of the sender that was preventing the transfer.
    TransferFailed = 5,
    /// The newly created contract is below the subsistence threshold after executing
    /// its constructor so no usable contract instance will be created.
    NewContractNotFunded = 6,
    /// No code could be found at the supplied code hash.
    CodeNotFound = 7,
    /// The account that was called is either no contract (e.g. user account) or is a tombstone.
    NotCallable = 8,
}

/// The raw return code returned by the host side.
#[repr(transparent)]
pub struct ReturnCode(u32);

impl ReturnCode {
    /// Returns the raw underlying `u32` representation.
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

fn extract_from_slice(output: &mut &mut [u8], new_len: usize) {
    debug_assert!(new_len <= output.len());
    let tmp = core::mem::take(output);
    *output = &mut tmp[..new_len];
}

pub fn instantiate(
    _code_hash: &[u8],
    _gas_limit: u64,
    _endowment: &[u8],
    _input: &[u8],
    _out_address: &mut &mut [u8],
    _out_return_value: &mut &mut [u8],
    _salt: &[u8],
) -> Result {
    unimplemented!("off-chain environment does not yet support `instantiate`");
}

pub fn call(
    _callee: &[u8],
    _gas_limit: u64,
    _value: &[u8],
    _input: &[u8],
    _output: &mut &mut [u8],
) -> Result {
    unimplemented!("off-chain environment does not yet support `call`");
}

pub fn transfer(_account_id: &[u8], _value: &[u8]) -> Result {
    unimplemented!("off-chain environment does not yet support `transfer`");
}

pub fn deposit_event(topics: &[u8], data: &[u8]) {
    // the first byte contains the number of topics in the slice
    let topics_count: scale::Compact<u32> =
        scale::Decode::decode(&mut &topics[0..1]).expect("decoding topics count failed");
    let topics_count = topics_count.0 as usize;

    let topics_vec = if topics_count > 0 {
        // the rest of the slice contains the topics
        let topics = &topics[1..];
        let bytes_per_topic = topics.len() / topics_count;
        let topics_vec: Vec<Vec<u8>> = topics
            .chunks(bytes_per_topic)
            .map(|chunk| chunk.to_vec())
            .collect();
        assert_eq!(topics_count, topics_vec.len());
        topics_vec
    } else {
        Vec::new()
    };

    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.emitted_events.push(EmittedEvent {
            topics: topics_vec,
            data: data.to_vec(),
        });
    })
}

pub fn set_storage(key: &[u8], encoded_value: &[u8]) {
    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.count_writes += 1;
    });

    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance
            .storage
            .insert(key.to_vec(), encoded_value.to_vec());
    })
}

pub fn clear_storage(key: &[u8]) {
    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.count_writes += 1;
    });

    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.storage.remove(key);
    })
}

pub fn get_storage(key: &[u8], output: &mut &mut [u8]) -> Result {
    REC_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance.count_reads += 1;
    });

    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        match instance.storage.get(key.to_vec().as_slice()) {
            Some(val) => {
                output[0..val.len()].copy_from_slice(val);
                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    })
}

/// Restores a tombstone to the original smart contract.
///
/// # Params
///
/// - `account_id`: Encoded bytes of the `AccountId` of the to-be-restored contract.
/// - `code_hash`: Encoded code hash of the to-be-restored contract.
/// - `rent_allowance`: The encoded rent allowance of the restored contract
///                     upon successful restoration.
/// - `filtered_keys`: Storage keys that will be ignored for the tombstone hash
///                    match calculation that decide whether the original contract
///                    storage and the storage of the restorer contract is equal.
pub fn restore_to(
    _account_id: &[u8],
    _code_hash: &[u8],
    _rent_allowance: &[u8],
    filtered_keys: &[Vec<u8>],
) {
    let _filtered_keys: Vec<crate::Key> = filtered_keys.iter().map(Into::into).collect();
    unimplemented!("off-chain environment does not yet support `restore_to`");
}

pub fn terminate(_beneficiary: &[u8]) -> ! {
    unimplemented!("off-chain environment does not yet support `terminate`");
}

pub fn call_chain_extension(
    _func_id: u32,
    _input: &[u8],
    _output: &mut &mut [u8],
) -> u32 {
    unimplemented!("off-chain environment does not yet support `call_chain_extension`");
}

pub fn input(_output: &mut &mut [u8]) {
    // TODO
    unimplemented!("off-chain environment does not yet support `input`");
}

/// The flags to indicate further information about the end of a contract execution.
pub struct ReturnFlags {
    _value: u32,
}

// impl is missing for `ReturnFlags`
pub fn return_value(flags: u32, _return_value: &[u8]) -> ! {
    // TODO
    let _flags = ReturnFlags { _value: flags };
    unimplemented!("off-chain environment does not yet support `return_value`");
}

macro_rules! impl_seal_wrapper_for {
    ( $( ($name:ident => $seal_name:ident), )* ) => {
        $(
            pub fn $name(_output: &mut &mut [u8]) {
                unimplemented!(
                    "off-chain environment does not yet support `{}`", stringify!($name)
                );
            }
        )*
    }
}

impl_seal_wrapper_for! {
    (block_number => seal_block_number),
    (balance => seal_balance), // TODO
    (gas_left => seal_gas_left),
    (value_transferred => seal_value_transferred), // TODO
    (now => seal_now),
    (rent_allowance => seal_rent_allowance),
    (minimum_balance => seal_minimum_balance),
    (tombstone_deposit => seal_tombstone_deposit),
}

pub fn caller(output: &mut &mut [u8]) {
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        let caller: Vec<u8> = instance
            .exec_context
            .as_ref()
            .expect("uninitialized context")
            .caller
            .clone()
            .into();
        output[..caller.len()].copy_from_slice(&caller[..]);
        extract_from_slice(output, caller.len());
    });
}

pub fn address(_output: &mut &mut [u8]) {
    // TODO
}

pub fn weight_to_fee(_gas: u64, _output: &mut &mut [u8]) {
    unimplemented!("off-chain environment does not yet support `weight_to_fee`");
}

pub fn set_rent_allowance(_value: &[u8]) {
    unimplemented!("off-chain environment does not yet support `set_rent_allowance`");
}

pub fn random(_subject: &[u8], _output: &mut &mut [u8]) {
    unimplemented!("off-chain environment does not yet support `random`");
}

pub fn println(content: &str) {
    println!("{}", content);
}

pub fn hash_blake2_256(input: &[u8], output: &mut [u8; 32]) {
    super::hashing::blake2b_256(input, output);
}

pub fn hash_blake2_128(input: &[u8], output: &mut [u8; 16]) {
    super::hashing::blake2b_128(input, output);
}

pub fn hash_sha2_256(input: &[u8], output: &mut [u8; 32]) {
    super::hashing::sha2_256(input, output);
}

pub fn hash_keccak_256(input: &[u8], output: &mut [u8; 32]) {
    super::hashing::keccak_256(input, output);
}
