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

use super::{
    hashing,
    Account,
    EnvInstance,
};
use crate::{
    call::{
        utils::ReturnType,
        CallParams,
        CreateParams,
    },
    hash::{
        Blake2x128,
        Blake2x256,
        CryptoHash,
        HashOutput,
        Keccak256,
        Sha2x256,
    },
    topics::Topics,
    EnvBackend,
    Environment,
    Error,
    Result,
    ReturnFlags,
    TypedEnvBackend,
};
use core::convert::TryInto;
use ink_primitives::Key;
use num_traits::Bounded;

const UNINITIALIZED_EXEC_CONTEXT: &str = "uninitialized execution context: \
a possible source of error could be that you are using `#[test]` instead of `#[ink::test]`.";

impl EnvInstance {
    /// Returns the callee account.
    fn callee_account(&self) -> &Account {
        let callee = self
            .exec_context()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .callee
            .clone();
        self.accounts
            .get_account_off(&callee)
            .expect("callee account does not exist")
    }

    /// Returns the callee account as mutable reference.
    fn callee_account_mut(&mut self) -> &mut Account {
        let callee = self
            .exec_context()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .callee
            .clone();
        self.accounts
            .get_account_off_mut(&callee)
            .expect("callee account does not exist")
    }
}

impl CryptoHash for Blake2x128 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 16];
        static_assertions::assert_type_eq_all!(
            <Blake2x128 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 16);
        hashing::blake2b_128(input, output);
    }
}

impl CryptoHash for Blake2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Blake2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 32);
        hashing::blake2b_256(input, output);
    }
}

impl CryptoHash for Sha2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Sha2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 32);
        hashing::sha2_256(input, output);
    }
}

impl CryptoHash for Keccak256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Keccak256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = arrayref::array_mut_ref!(output, 0, 32);
        hashing::keccak_256(input, output);
    }
}

impl EnvBackend for EnvInstance {
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V)
    where
        V: scale::Encode,
    {
        self.callee_account_mut()
            .set_storage(*key, value)
            .expect("callee account is not a smart contract");
    }

    fn get_contract_storage<R>(&mut self, key: &Key) -> Result<Option<R>>
    where
        R: scale::Decode,
    {
        self.callee_account()
            .get_storage::<R>(*key)
            .map_err(Into::into)
    }

    fn clear_contract_storage(&mut self, key: &Key) {
        if !self.clear_storage_disabled {
            self.callee_account_mut()
                .clear_storage(*key)
                .expect("callee account is not a smart contract");
        }
    }

    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode,
    {
        self.exec_context()
            .map(|exec_ctx| &exec_ctx.call_data)
            .map(scale::Encode::encode)
            .map_err(Into::into)
            .and_then(|encoded| {
                <T as scale::Decode>::decode(&mut &encoded[..])
                    .map_err(|_| scale::Error::from("could not decode input call data"))
                    .map_err(Into::into)
            })
    }

    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: scale::Encode,
    {
        let ctx = self.exec_context_mut().expect(UNINITIALIZED_EXEC_CONTEXT);
        ctx.output = Some(return_value.encode());
        std::process::exit(flags.into_u32() as i32)
    }

    fn debug_message(&mut self, message: &str) {
        self.debug_buf.debug_message(message)
    }

    fn hash_bytes<H>(&mut self, input: &[u8], output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
    {
        <H as CryptoHash>::hash(input, output)
    }

    fn hash_encoded<H, T>(&mut self, input: &T, output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
        T: scale::Encode,
    {
        let encoded = input.encode();
        self.hash_bytes::<H>(&encoded[..], output)
    }

    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()> {
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
            Err(_) => Err(Error::EcdsaRecoverFailed),
        }
    }

    fn call_chain_extension<I, T, E, ErrorCode, F, D>(
        &mut self,
        func_id: u32,
        input: &I,
        status_to_result: F,
        decode_to_result: D,
    ) -> ::core::result::Result<T, E>
    where
        I: scale::Encode,
        T: scale::Decode,
        E: From<ErrorCode>,
        F: FnOnce(u32) -> ::core::result::Result<(), ErrorCode>,
        D: FnOnce(&[u8]) -> ::core::result::Result<T, E>,
    {
        let encoded_input = input.encode();
        let (status_code, output) = self
            .chain_extension_handler
            .eval(func_id, &encoded_input)
            .expect("encountered unexpected missing chain extension method");
        status_to_result(status_code)?;
        let decoded = decode_to_result(output)?;
        Ok(decoded)
    }
}

impl EnvInstance {
    fn transfer_impl<T>(
        &mut self,
        destination: &T::AccountId,
        value: T::Balance,
    ) -> Result<()>
    where
        T: Environment,
    {
        let src_id = self.account_id::<T>();
        let src_value = self
            .accounts
            .get_account::<T>(&src_id)
            .expect("account of executed contract must exist")
            .balance::<T>()?;
        if src_value < value {
            return Err(Error::TransferFailed)
        }
        let dst_value = self
            .accounts
            .get_or_create_account::<T>(destination)
            .balance::<T>()?;
        self.accounts
            .get_account_mut::<T>(&src_id)
            .expect("account of executed contract must exist")
            .set_balance::<T>(src_value - value)?;
        self.accounts
            .get_account_mut::<T>(destination)
            .expect("the account must exist already or has just been created")
            .set_balance::<T>(dst_value + value)?;
        Ok(())
    }

    // Remove the calling account and transfer remaining balance.
    //
    // This function never returns. Either the termination was successful and the
    // execution of the destroyed contract is halted. Or it failed during the termination
    // which is considered fatal.
    fn terminate_contract_impl<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: Environment,
    {
        // Send the remaining balance to the beneficiary
        let all: T::Balance = self.balance::<T>();
        self.transfer_impl::<T>(&beneficiary, all)
            .expect("transfer did not work ");

        // Remove account
        let contract_id = self.account_id::<T>();
        self.accounts.remove_account::<T>(contract_id);

        // Encode the result of the termination and panic with it.
        // This enables testing for the proper result and makes sure this
        // method returns `Never`.
        let res = crate::test::ContractTerminationResult::<T> {
            beneficiary,
            transferred: all,
        };
        std::panic::panic_any(scale::Encode::encode(&res));
    }
}

impl TypedEnvBackend for EnvInstance {
    fn caller<T: Environment>(&mut self) -> T::AccountId {
        self.exec_context()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .caller::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `caller` property: {:?}", error)
            })
    }

    fn transferred_value<T: Environment>(&mut self) -> T::Balance {
        self.exec_context()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .transferred_value::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `transferred_value` property: {:?}", error)
            })
    }

    /// Emulates gas price calculation
    fn weight_to_fee<T: Environment>(&mut self, gas: u64) -> T::Balance {
        use crate::arithmetic::Saturating as _;

        let gas_price = self.chain_spec.gas_price::<T>().unwrap_or_else(|error| {
            panic!("could not read `gas_price` property: {:?}", error)
        });
        gas_price.saturating_mul(gas.try_into().unwrap_or_else(|_| Bounded::max_value()))
    }

    fn gas_left<T: Environment>(&mut self) -> u64 {
        self.exec_context()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .gas::<T>()
    }

    fn block_timestamp<T: Environment>(&mut self) -> T::Timestamp {
        self.current_block()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .timestamp::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `block_timestamp` property: {:?}", error)
            })
    }

    fn account_id<T: Environment>(&mut self) -> T::AccountId {
        self.exec_context()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .callee::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `account_id` property: {:?}", error)
            })
    }

    fn balance<T: Environment>(&mut self) -> T::Balance {
        self.callee_account()
            .balance::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `balance` property: {:?}", error)
            })
    }

    fn block_number<T: Environment>(&mut self) -> T::BlockNumber {
        self.current_block()
            .expect(UNINITIALIZED_EXEC_CONTEXT)
            .number::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `block_number` property: {:?}", error)
            })
    }

    fn minimum_balance<T: Environment>(&mut self) -> T::Balance {
        self.chain_spec
            .minimum_balance::<T>()
            .unwrap_or_else(|error| {
                panic!("could not read `minimum_balance` property: {:?}", error)
            })
    }

    fn emit_event<T, Event>(&mut self, new_event: Event)
    where
        T: Environment,
        Event: Topics + scale::Encode,
    {
        self.emitted_events.record::<T, Event>(new_event)
    }

    fn invoke_contract<T, Args>(&mut self, params: &CallParams<T, Args, ()>) -> Result<()>
    where
        T: Environment,
        Args: scale::Encode,
    {
        let _gas_limit = params.gas_limit();
        let _callee = params.callee();
        let _call_flags = params.call_flags().into_u32();
        let _transferred_value = params.transferred_value();
        let _input = params.exec_input();
        unimplemented!("off-chain environment does not support contract invocation")
    }

    fn eval_contract<T, Args, R>(
        &mut self,
        _call_params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        T: Environment,
        Args: scale::Encode,
        R: scale::Decode,
    {
        unimplemented!("off-chain environment does not support contract evaluation")
    }

    fn instantiate_contract<T, Args, Salt, C>(
        &mut self,
        params: &CreateParams<T, Args, Salt, C>,
    ) -> Result<T::AccountId>
    where
        T: Environment,
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        let _code_hash = params.code_hash();
        let _gas_limit = params.gas_limit();
        let _endowment = params.endowment();
        let _input = params.exec_input();
        let _salt_bytes = params.salt_bytes();
        unimplemented!("off-chain environment does not support contract instantiation")
    }

    fn terminate_contract<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: Environment,
    {
        self.terminate_contract_impl::<T>(beneficiary)
    }

    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: Environment,
    {
        self.transfer_impl::<T>(&destination, value)
    }

    fn random<T>(&mut self, subject: &[u8]) -> Result<(T::Hash, T::BlockNumber)>
    where
        T: Environment,
    {
        let block = self.current_block().expect(UNINITIALIZED_EXEC_CONTEXT);
        Ok((block.random::<T>(subject)?, block.number::<T>()?))
    }
}
