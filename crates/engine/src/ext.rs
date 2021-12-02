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
//! See [the documentation for the `contract` module](https://docs.rs/crate/pallet-contracts)
//! for more information.

use crate::{
    database::Database,
    exec_context::ExecContext,
    test_api::{
        DebugInfo,
        EmittedEvent,
    },
    types::AccountId,
};
use std::panic::panic_any;

type Result = core::result::Result<(), Error>;

macro_rules! define_error_codes {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident = $discr:literal,
        )*
    ) => {
        /// Every error that can be returned to a contract when it calls any of the host functions.
        #[cfg_attr(test, derive(PartialEq, Eq))]
        #[derive(Debug)]
        #[repr(u32)]
        pub enum Error {
            $(
                $( #[$attr] )*
                $name = $discr,
            )*
            /// Returns if an unknown error was received from the host module.
            Unknown,
        }

        impl From<ReturnCode> for Result {
            #[inline]
            fn from(return_code: ReturnCode) -> Self {
                match return_code.0 {
                    0 => Ok(()),
                    $(
                        $discr => Err(Error::$name),
                    )*
                    _ => Err(Error::Unknown),
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
    /// The account that was called is no contract, but a plain account.
    NotCallable = 8,
    /// The call to `seal_debug_message` had no effect because debug message
    /// recording was disabled.
    LoggingDisabled = 9,
    /// ECDSA pubkey recovery failed. Most probably wrong recovery id or signature.
    EcdsaRecoveryFailed = 11,
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

/// The off-chain engine.
pub struct Engine {
    /// The environment database.
    pub database: Database,
    /// The current execution context.
    pub exec_context: ExecContext,
    /// Recorder for relevant interactions with the engine.
    /// This is specifically about debug info. This info is
    /// not available in the `contracts` pallet.
    pub(crate) debug_info: DebugInfo,
}

impl Engine {
    // Creates a new `Engine instance.
    pub fn new() -> Self {
        Self {
            database: Database::new(),
            exec_context: ExecContext::new(),
            debug_info: DebugInfo::new(),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Transfers value from the contract to the destination account.
    pub fn transfer(&mut self, account_id: &[u8], mut value: &[u8]) -> Result {
        // Note that a transfer of `0` is allowed here
        let increment = <u128 as scale::Decode>::decode(&mut value)
            .map_err(|_| Error::TransferFailed)?;

        let dest = account_id.to_vec();
        // Note that the destination account does not have to exist
        let dest_old_balance = self.get_balance(dest.clone()).unwrap_or_default();

        let contract = self.get_callee();
        let contract_old_balance = self
            .get_balance(contract.clone())
            .map_err(|_| Error::TransferFailed)?;

        self.database
            .set_balance(&contract, contract_old_balance - increment);
        self.database
            .set_balance(&dest, dest_old_balance + increment);
        Ok(())
    }

    /// Deposits an event identified by the supplied topics and data.
    pub fn deposit_event(&mut self, topics: &[u8], data: &[u8]) {
        // The first byte contains the number of topics in the slice
        let topics_count: scale::Compact<u32> = scale::Decode::decode(&mut &topics[0..1])
            .expect("decoding number of topics failed");
        let topics_count = topics_count.0 as usize;

        let topics_vec = if topics_count > 0 {
            // The rest of the slice contains the topics
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

        self.debug_info.record_event(EmittedEvent {
            topics: topics_vec,
            data: data.to_vec(),
        });
    }

    /// Writes the encoded value into the storage at the given key.
    pub fn set_storage(&mut self, key: &[u8; 32], encoded_value: &[u8]) {
        let callee = self.get_callee();
        let account_id = AccountId::from_bytes(&callee[..]);

        self.debug_info.inc_writes(account_id.clone());
        self.debug_info
            .record_cell_for_account(account_id, key.to_vec());

        // We ignore if storage is already set for this key
        let _ = self.database.insert_into_contract_storage(
            &callee,
            key,
            encoded_value.to_vec(),
        );
    }

    /// Returns the decoded contract storage at the key if any.
    pub fn get_storage(&mut self, key: &[u8; 32], output: &mut &mut [u8]) -> Result {
        let callee = self.get_callee();
        let account_id = AccountId::from_bytes(&callee[..]);

        self.debug_info.inc_reads(account_id);
        match self.database.get_from_contract_storage(&callee, key) {
            Some(val) => {
                set_output(output, val);
                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }

    /// Removes the storage entries at the given key.
    pub fn clear_storage(&mut self, key: &[u8; 32]) {
        let callee = self.get_callee();
        let account_id = AccountId::from_bytes(&callee[..]);
        self.debug_info.inc_writes(account_id.clone());
        let _ = self
            .debug_info
            .remove_cell_for_account(account_id, key.to_vec());
        let _ = self.database.remove_contract_storage(&callee, key);
    }

    /// Remove the calling account and transfer remaining balance.
    ///
    /// This function never returns. Either the termination was successful and the
    /// execution of the destroyed contract is halted. Or it failed during the
    /// termination which is considered fatal.
    pub fn terminate(&mut self, beneficiary: &[u8]) -> ! {
        // Send the remaining balance to the beneficiary
        let contract = self.get_callee();
        let all = self.get_balance(contract).expect("could not get balance");
        let value = &scale::Encode::encode(&all)[..];
        self.transfer(beneficiary, value)
            .expect("transfer did not work");

        // Encode the result of the termination and panic with it.
        // This enables testing for the proper result and makes sure this
        // method returns `Never`.
        let res = (all, beneficiary.to_vec());
        panic_any(scale::Encode::encode(&res));
    }

    /// Returns the address of the caller.
    pub fn caller(&self, output: &mut &mut [u8]) {
        let caller = self
            .exec_context
            .caller
            .as_ref()
            .expect("no caller has been set")
            .as_bytes();
        set_output(output, caller);
    }

    /// Returns the balance of the executed contract.
    pub fn balance(&self, output: &mut &mut [u8]) {
        let contract = self
            .exec_context
            .callee
            .as_ref()
            .expect("no callee has been set");

        let balance_in_storage = self
            .database
            .get_balance(&contract.as_bytes().to_vec())
            .expect("currently executing contract must exist");
        let balance = scale::Encode::encode(&balance_in_storage);
        set_output(output, &balance[..])
    }

    /// Returns the transferred value for the called contract.
    pub fn value_transferred(&self, output: &mut &mut [u8]) {
        let value_transferred: Vec<u8> =
            scale::Encode::encode(&self.exec_context.value_transferred);
        set_output(output, &value_transferred[..])
    }

    /// Returns the address of the executed contract.
    pub fn address(&self, output: &mut &mut [u8]) {
        let callee = self
            .exec_context
            .callee
            .as_ref()
            .expect("no callee has been set")
            .as_bytes();
        set_output(output, callee)
    }

    /// Records the given debug message and appends to stdout.
    pub fn debug_message(&mut self, message: &str) {
        self.debug_info.record_debug_message(String::from(message));
        print!("{}", message);
    }

    /// Conduct the BLAKE-2 256-bit hash and place the result into `output`.
    pub fn hash_blake2_256(input: &[u8], output: &mut [u8; 32]) {
        super::hashing::blake2b_256(input, output);
    }

    /// Conduct the BLAKE-2 128-bit hash and place the result into `output`.
    pub fn hash_blake2_128(input: &[u8], output: &mut [u8; 16]) {
        super::hashing::blake2b_128(input, output);
    }

    /// Conduct the SHA-2 256-bit hash and place the result into `output`.
    pub fn hash_sha2_256(input: &[u8], output: &mut [u8; 32]) {
        super::hashing::sha2_256(input, output);
    }

    /// Conduct the KECCAK 256-bit hash and place the result into `output`.
    pub fn hash_keccak_256(input: &[u8], output: &mut [u8; 32]) {
        super::hashing::keccak_256(input, output);
    }

    pub fn block_number(&self, _output: &mut &mut [u8]) {
        unimplemented!("off-chain environment does not yet support `block_number`");
    }

    pub fn block_timestamp(&self, _output: &mut &mut [u8]) {
        unimplemented!("off-chain environment does not yet support `block_timestamp`");
    }

    pub fn gas_left(&self, _output: &mut &mut [u8]) {
        unimplemented!("off-chain environment does not yet support `gas_left`");
    }

    pub fn minimum_balance(&self, _output: &mut &mut [u8]) {
        unimplemented!("off-chain environment does not yet support `minimum_balance`");
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        &mut self,
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
        &mut self,
        _callee: &[u8],
        _gas_limit: u64,
        _value: &[u8],
        _input: &[u8],
        _output: &mut &mut [u8],
    ) -> Result {
        unimplemented!("off-chain environment does not yet support `call`");
    }

    pub fn weight_to_fee(&self, _gas: u64, _output: &mut &mut [u8]) {
        unimplemented!("off-chain environment does not yet support `weight_to_fee`");
    }

    pub fn random(&self, _subject: &[u8], _output: &mut &mut [u8]) {
        unimplemented!("off-chain environment does not yet support `random`");
    }

    pub fn call_chain_extension(
        &mut self,
        _func_id: u32,
        _input: &[u8],
        _output: &mut &mut [u8],
    ) -> u32 {
        unimplemented!(
            "off-chain environment does not yet support `call_chain_extension`"
        );
    }

    /// Recovers the compressed ECDSA public key for given `signature` and `message_hash`,
    /// and stores the result in `output`.
    pub fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result {
        use secp256k1::{
            recovery::{
                RecoverableSignature,
                RecoveryId,
            },
            Message,
            Secp256k1,
        };

        // In most implementations, the v is just 0 or 1 internally, but 27 was added
        // as an arbitrary number for signing Bitcoin messages and Ethereum adopted that as well.
        let recovery_byte = if signature[64] > 26 {
            signature[64] - 27
        } else {
            signature[64]
        };

        let recovery_id = RecoveryId::from_i32(recovery_byte as i32)
            .unwrap_or_else(|error| panic!("Unable to parse the recovery id: {}", error));

        let message = Message::from_slice(message_hash).unwrap_or_else(|error| {
            panic!("Unable to create the message from hash: {}", error)
        });
        let signature =
            RecoverableSignature::from_compact(&signature[0..64], recovery_id)
                .unwrap_or_else(|error| {
                    panic!("Unable to parse the signature: {}", error)
                });

        let secp = Secp256k1::new();
        let pub_key = secp.recover(&message, &signature);
        match pub_key {
            Ok(pub_key) => {
                *output = pub_key.serialize();
                Ok(())
            }
            Err(_) => Err(Error::EcdsaRecoveryFailed),
        }
    }
}

/// Copies the `slice` into `output`.
///
/// Panics if the slice is too large and does not fit.
fn set_output(output: &mut &mut [u8], slice: &[u8]) {
    assert!(
        slice.len() <= output.len(),
        "the output buffer is too small! the decoded storage is of size {} bytes, \
        but the output buffer has only room for {}.",
        slice.len(),
        output.len(),
    );
    output[..slice.len()].copy_from_slice(slice);
}
