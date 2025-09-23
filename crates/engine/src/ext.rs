// Copyright (C) Use Ink (UK) Ltd.
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

//! Provides the same interface as Substrate's FRAME `revive` module.
//!
//! See [the documentation for the `revive` module](https://docs.rs/crate/pallet-revive)
//! for more information.

use crate::{
    database::Database,
    exec_context::ExecContext,
    test_api::{
        DebugInfo,
        EmittedEvent,
    },
    types::BlockTimestamp,
};
use hex_literal::hex;
use ink_primitives::{
    Address,
    U256,
};
pub use pallet_revive_uapi::ReturnErrorCode as Error;
use std::panic::panic_any;

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
    /// The chain specification.
    pub chain_spec: ChainSpec,
}

/// The chain specification.
pub struct ChainSpec {
    /// The current gas price.
    pub gas_price: U256,
    /// The minimum value an account of the chain must have
    /// (i.e. the chain's existential deposit).
    pub minimum_balance: U256,
    /// The targeted block time.
    pub block_time: BlockTimestamp,
}

/// The default values for the chain specification are:
///
///   * `gas_price`: 100
///   * `minimum_balance`: 42
///   * `block_time`: 6
///
/// There is no particular reason behind choosing them this way.
impl Default for ChainSpec {
    fn default() -> Self {
        Self {
            gas_price: 100.into(),
            minimum_balance: 42.into(),
            block_time: 6,
        }
    }
}

impl Engine {
    // Creates a new `Engine` instance.
    pub fn new() -> Self {
        Self {
            database: Database::new(),
            exec_context: ExecContext::new(),
            debug_info: DebugInfo::new(),
            chain_spec: ChainSpec::default(),
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
    #[allow(clippy::arithmetic_side_effects)] // todo
    pub fn transfer(&mut self, dest: Address, mut value: &[u8]) -> Result<(), Error> {
        // Note that a transfer of `0` is allowed here
        let increment = <u128 as scale::Decode>::decode(&mut value)
            .map_err(|_| Error::TransferFailed)?;

        // Note that the destination account does not have to exist
        let dest_old_balance = self.get_balance(dest).unwrap_or_default();

        let contract = self.get_callee();
        let contract_old_balance = self
            .get_balance(contract)
            .map_err(|_| Error::TransferFailed)?;

        self.database
            .set_balance(&contract, contract_old_balance - increment);
        self.database
            .set_balance(&dest, dest_old_balance + increment);
        Ok(())
    }

    /// Deposits an event identified by the supplied topics and data.
    pub fn deposit_event(&mut self, topics: &[[u8; 32]], data: &[u8]) {
        self.debug_info.record_event(EmittedEvent {
            topics: topics.to_vec(),
            data: data.to_vec(),
        });
    }

    /// Writes the encoded value into the storage at the given key.
    /// Returns the size of the previously stored value at the key if any.
    pub fn set_storage(&mut self, key: &[u8], encoded_value: &[u8]) -> Option<u32> {
        let callee = self.get_callee();

        self.debug_info.inc_writes(callee);
        self.debug_info
            .record_cell_for_account(callee, key.to_vec());

        self.database
            .insert_into_contract_storage(&callee, key, encoded_value.to_vec())
            .map(|v| <u32>::try_from(v.len()).expect("usize to u32 conversion failed"))
    }

    /// Returns the contract storage bytes at the key if any.
    pub fn get_storage(&mut self, key: &[u8]) -> Result<&[u8], Error> {
        let callee = self.get_callee();

        self.debug_info.inc_reads(callee);
        match self.database.get_from_contract_storage(&callee, key) {
            Some(val) => Ok(val),
            None => Err(Error::KeyNotFound),
        }
    }

    /// Removes the storage entries at the given key,
    /// returning previously stored value at the key if any.
    pub fn take_storage(&mut self, key: &[u8]) -> Result<Vec<u8>, Error> {
        let callee = self.get_callee();

        self.debug_info.inc_writes(callee);
        match self.database.remove_contract_storage(&callee, key) {
            Some(val) => Ok(val),
            None => Err(Error::KeyNotFound),
        }
    }

    /// Returns the size of the value stored in the contract storage at the key if any.
    pub fn contains_storage(&mut self, key: &[u8]) -> Option<u32> {
        let callee = self.get_callee();

        self.debug_info.inc_reads(callee);
        self.database
            .get_from_contract_storage(&callee, key)
            .map(|val| val.len() as u32)
    }

    /// Removes the storage entries at the given key.
    /// Returns the size of the previously stored value at the key if any.
    pub fn clear_storage(&mut self, key: &[u8]) -> Option<u32> {
        let callee = self.get_callee();
        self.debug_info.inc_writes(callee);
        let _ = self
            .debug_info
            .remove_cell_for_account(callee, key.to_vec());
        self.database
            .remove_contract_storage(&callee, key)
            .map(|val| val.len() as u32)
    }

    /// Remove the calling account and transfer remaining balance.
    ///
    /// todo is the following comment still correct?
    /// This function never returns. Either the termination was successful and the
    /// execution of the destroyed contract is halted. Or it failed during the
    /// termination which is considered fatal.
    pub fn terminate(&mut self, beneficiary: Address) -> ! {
        // Send the remaining balance to the beneficiary
        let contract = self.get_callee();
        let all = self
            .get_balance(contract)
            .unwrap_or_else(|err| panic!("could not get balance: {err:?}"));
        let value = &scale::Encode::encode(&all)[..];
        self.transfer(beneficiary, value)
            .unwrap_or_else(|err| panic!("transfer did not work: {err:?}"));

        // Encode the result of the termination and panic with it.
        // This enables testing for the proper result and makes sure this
        // method returns `Never`.
        let res = (all, beneficiary);
        panic_any(scale::Encode::encode(&res));
    }

    /// Returns the address of the caller.
    pub fn caller(&self, output: &mut &mut [u8]) {
        let caller = self.exec_context.caller;
        let caller = scale::Encode::encode(&caller);
        set_output(output, &caller[..])
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
            .get_balance(contract)
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

    pub fn account_id(&self, output: &mut &mut [u8]) {
        let callee = self
            .exec_context
            .callee
            .as_ref()
            .expect("no callee has been set");
        let account_id = self.database.to_account_id(callee);
        set_output(output, account_id.as_slice())
    }

    /// Retrieves the account id for a specified address.
    pub fn to_account_id(&self, input: &[u8], output: &mut &mut [u8]) {
        let addr =
            scale::Decode::decode(&mut &input[..]).expect("unable to decode Address");
        let account_id = self.database.to_account_id(&addr);
        set_output(output, account_id.as_slice())
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

    /// Returns the current block number.
    pub fn block_number(&self, output: &mut &mut [u8]) {
        let block_number: Vec<u8> =
            scale::Encode::encode(&self.exec_context.block_number);
        set_output(output, &block_number[..])
    }

    /// Returns the timestamp of the current block.
    pub fn block_timestamp(&self, output: &mut &mut [u8]) {
        let block_timestamp: Vec<u8> =
            scale::Encode::encode(&self.exec_context.block_timestamp);
        set_output(output, &block_timestamp[..])
    }

    /// Returns the minimum balance that is required for creating an account
    /// (i.e. the chain's existential deposit).
    pub fn minimum_balance(&self, output: &mut &mut [u8]) {
        let minimum_balance: Vec<u8> =
            scale::Encode::encode(&self.chain_spec.minimum_balance);
        set_output(output, &minimum_balance[..])
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
    ) -> Result<(), Error> {
        unimplemented!("off-chain environment does not yet support `instantiate`");
    }

    pub fn call(
        &mut self,
        callee: &[u8],
        _gas_limit: u64,
        _value: &[u8],
        input: &[u8],
        output: &mut &mut [u8],
    ) -> Result<(), Error> {
        const ECRECOVER: [u8; 20] = hex!("0000000000000000000000000000000000000001");
        if callee == ECRECOVER {
            let mut signature = [0u8; 65];
            signature.copy_from_slice(&input[..65]);
            let mut message_hash = [0u8; 32];
            message_hash.copy_from_slice(&input[65..65 + 32]);

            let out: &mut [u8; 33] = output
                .as_mut()
                .try_into()
                .expect("Slice must be exactly 33 bytes long");
            let _ = self.ecdsa_recover(&signature, &message_hash, out);
        }
        unimplemented!(
            "off-chain environment does not yet support `call` for non-precompiles"
        );
    }

    /// Emulates gas price calculation.
    pub fn weight_to_fee(&self, gas: u64, output: &mut &mut [u8]) {
        let fee = self.chain_spec.gas_price.saturating_mul(gas.into());
        let fee: Vec<u8> = scale::Encode::encode(&fee);
        set_output(output, &fee[..])
    }
}

impl Engine {
    /// Recovers the compressed ECDSA public key for given `signature` and `message_hash`,
    /// and stores the result in `output`.
    #[allow(clippy::arithmetic_side_effects)] // todo
    pub fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<(), Error> {
        use secp256k1::{
            Message,
            SECP256K1,
            ecdsa::{
                RecoverableSignature,
                RecoveryId,
            },
        };

        // In most implementations, the v is just 0 or 1 internally, but 27 was added
        // as an arbitrary number for signing Bitcoin messages and Ethereum adopted that
        // as well.
        let recovery_byte = if signature[64] > 26 {
            signature[64] - 27
        } else {
            signature[64]
        };

        let recovery_id = RecoveryId::try_from(recovery_byte as i32)
            .unwrap_or_else(|error| panic!("Unable to parse the recovery id: {error}"));

        let message = Message::from_digest_slice(message_hash).unwrap_or_else(|error| {
            panic!("Unable to create the message from hash: {error}")
        });
        let signature =
            RecoverableSignature::from_compact(&signature[0..64], recovery_id)
                .unwrap_or_else(|error| panic!("Unable to parse the signature: {error}"));

        let pub_key = SECP256K1.recover_ecdsa(&message, &signature);
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
