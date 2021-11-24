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

use crate::{
    call::{
        utils::ReturnType,
        CallParams,
        CreateParams,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    topics::Topics,
    types::{
        RentParams,
        RentStatus,
    },
    Environment,
    Result,
};
use ink_primitives::Key;

/// The flags to indicate further information about the end of a contract execution.
#[derive(Default)]
pub struct ReturnFlags {
    value: u32,
}

impl ReturnFlags {
    /// Sets the bit to indicate that the execution is going to be reverted.
    pub fn set_reverted(mut self, has_reverted: bool) -> Self {
        match has_reverted {
            true => self.value |= has_reverted as u32,
            false => self.value &= !has_reverted as u32,
        }
        self
    }

    /// Returns the underlying `u32` representation.
    #[cfg(not(feature = "ink-experimental-engine"))]
    pub(crate) fn into_u32(self) -> u32 {
        self.value
    }
}

/// The flags used to change the behavior of a contract call.
#[derive(Copy, Clone, Debug, Default)]
pub struct CallFlags {
    forward_input: bool,
    clone_input: bool,
    tail_call: bool,
    allow_reentry: bool,
}

impl CallFlags {
    /// Forwards the input for the current function to the callee.
    ///
    /// # Note
    ///
    /// A forwarding call will consume the current contracts input. Any attempt to
    /// access the input after this call returns (e.g. by trying another forwarding call)
    /// will lead to a contract revert.
    /// Consider using [`Self::set_clone_input`] in order to preserve the input.
    pub const fn set_forward_input(mut self, forward_input: bool) -> Self {
        self.forward_input = forward_input;
        self
    }

    /// Identical to [`Self::set_forward_input`] but without consuming the input.
    ///
    /// This adds some additional weight costs to the call.
    ///
    /// # Note
    ///
    /// This implies [`Self::set_forward_input`] and takes precedence when both are set.
    pub const fn set_clone_input(mut self, clone_input: bool) -> Self {
        self.clone_input = clone_input;
        self
    }

    /// Do not return from the call but rather return the result of the callee to the
    /// callers caller.
    ///
    /// # Note
    ///
    /// This makes the current contract completely transparent to its caller by replacing
    /// this contracts potential output with the callee ones. Any code after the contract
    /// calls has been invoked can be safely considered unreachable.
    pub const fn set_tail_call(mut self, tail_call: bool) -> Self {
        self.tail_call = tail_call;
        self
    }

    /// Allow the callee to reenter into the current contract.
    ///
    /// Without this flag any reentrancy into the current contract that originates from
    /// the callee (or any of its callees) is denied. This includes the first callee:
    /// You cannot call into yourself with this flag set.
    pub const fn set_allow_reentry(mut self, allow_reentry: bool) -> Self {
        self.allow_reentry = allow_reentry;
        self
    }

    /// Returns the underlying `u32` representation of the call flags.
    ///
    /// This value is used to forward the call flag information to the
    /// `contracts` pallet.
    pub(crate) const fn into_u32(self) -> u32 {
        self.forward_input as u32
            | ((self.clone_input as u32) << 1)
            | ((self.tail_call as u32) << 2)
            | ((self.allow_reentry as u32) << 3)
    }

    /// Returns `true` if input forwarding is set.
    ///
    /// # Note
    ///
    /// See [`Self::set_forward_input`] for more information.
    pub const fn forward_input(&self) -> bool {
        self.forward_input
    }

    /// Returns `true` if input cloning is set.
    ///
    /// # Note
    ///
    /// See [`Self::set_clone_input`] for more information.
    pub const fn clone_input(&self) -> bool {
        self.clone_input
    }

    /// Returns `true` if the tail call property is set.
    ///
    /// # Note
    ///
    /// See [`Self::set_tail_call`] for more information.
    pub const fn tail_call(&self) -> bool {
        self.tail_call
    }

    /// Returns `true` if call reentry is allowed.
    ///
    /// # Note
    ///
    /// See [`Self::set_allow_reentry`] for more information.
    pub const fn allow_reentry(&self) -> bool {
        self.allow_reentry
    }
}

/// Environmental contract functionality that does not require `Environment`.
pub trait EnvBackend {
    /// Writes the value to the contract storage under the given key.
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V)
    where
        V: scale::Encode;

    /// Returns the value stored under the given key in the contract's storage if any.
    ///
    /// # Errors
    ///
    /// - If the decoding of the typed value failed
    fn get_contract_storage<R>(&mut self, key: &Key) -> Result<Option<R>>
    where
        R: scale::Decode;

    /// Clears the contract's storage key entry.
    fn clear_contract_storage(&mut self, key: &Key);

    /// Returns the execution input to the executed contract and decodes it as `T`.
    ///
    /// # Note
    ///
    /// - The input is the 4-bytes selector followed by the arguments
    ///   of the called function in their SCALE encoded representation.
    /// - No prior interaction with the environment must take place before
    ///   calling this procedure.
    ///
    /// # Usage
    ///
    /// Normally contracts define their own `enum` dispatch types respective
    /// to their exported constructors and messages that implement `scale::Decode`
    /// according to the constructors or messages selectors and their arguments.
    /// These `enum` dispatch types are then given to this procedure as the `T`.
    ///
    /// When using ink! users do not have to construct those enum dispatch types
    /// themselves as they are normally generated by the ink! code generation
    /// automatically.
    ///
    /// # Errors
    ///
    /// If the given `T` cannot be properly decoded from the expected input.
    fn decode_input<T>(&mut self) -> Result<T>
    where
        T: scale::Decode;

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// Calling this method will end contract execution immediately.
    /// It will return the given return value back to its caller.
    ///
    /// The `flags` parameter can be used to revert the state changes of the
    /// entire execution if necessary.
    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: scale::Encode;

    /// Emit a custom debug message.
    ///
    /// The message is appended to the debug buffer which is then supplied to the calling RPC
    /// client. This buffer is also printed as a debug message to the node console if the
    /// `debug` log level is enabled for the `runtime::contracts` target.
    ///
    /// If debug message recording is disabled in the contracts pallet, which is always the case
    /// when the code is executing on-chain, then this will have no effect.
    fn debug_message(&mut self, content: &str);

    /// Conducts the crypto hash of the given input and stores the result in `output`.
    fn hash_bytes<H>(&mut self, input: &[u8], output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash;

    /// Conducts the crypto hash of the given encoded input and stores the result in `output`.
    fn hash_encoded<H, T>(&mut self, input: &T, output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
        T: scale::Encode;

    /// Recovers the compressed ECDSA public key for given `signature` and `message_hash`,
    /// and stores the result in `output`.
    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()>;

    /// Low-level interface to call a chain extension method.
    ///
    /// Returns the output of the chain extension of the specified type.
    ///
    /// # Errors
    ///
    /// - If the chain extension with the given ID does not exist.
    /// - If the inputs had an unexpected encoding.
    /// - If the output could not be properly decoded.
    /// - If some extension specific condition has not been met.
    ///
    /// # Developer Note
    ///
    /// A valid implementation applies the `status_to_result` closure on
    /// the status code returned by the actual call to the chain extension
    /// method.
    /// Only if the closure finds that the given status code indicates a
    /// successful call to the chain extension method is the resulting
    /// output buffer passed to the `decode_to_result` closure, in order to
    /// drive the decoding and error management process from the outside.
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
        D: FnOnce(&[u8]) -> ::core::result::Result<T, E>;
}

/// Environmental contract functionality.
pub trait TypedEnvBackend: EnvBackend {
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`caller`][`crate::caller`]
    fn caller<T: Environment>(&mut self) -> T::AccountId;

    /// Returns the transferred balance for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`transferred_value`][`crate::transferred_value`]
    fn transferred_value<T: Environment>(&mut self) -> T::Balance;

    /// Returns the price for the specified amount of gas.
    ///
    /// # Note
    ///
    /// For more details visit: [`weight_to_fee`][`crate::weight_to_fee`]
    fn weight_to_fee<T: Environment>(&mut self, gas: u64) -> T::Balance;

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`gas_left`][`crate::gas_left`]
    fn gas_left<T: Environment>(&mut self) -> u64;

    /// Returns the timestamp of the current block.
    ///
    /// # Note
    ///
    /// For more details visit: [`block_timestamp`][`crate::block_timestamp`]
    fn block_timestamp<T: Environment>(&mut self) -> T::Timestamp;

    /// Returns the address of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`account_id`][`crate::account_id`]
    fn account_id<T: Environment>(&mut self) -> T::AccountId;

    /// Returns the balance of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`balance`][`crate::balance`]
    fn balance<T: Environment>(&mut self) -> T::Balance;

    /// Returns the current rent allowance for the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`rent_allowance`][`crate::rent_allowance`]
    fn rent_allowance<T: Environment>(&mut self) -> T::Balance;

    /// Returns information needed for rent calculations.
    ///
    /// # Note
    ///
    /// For more details visit: [`RentParams`][`crate::RentParams`]
    fn rent_params<T: Environment>(&mut self) -> Result<RentParams<T>>;

    /// Returns information about the required deposit and resulting rent.
    ///
    /// # Note
    ///
    /// For more details visit: [`RentStatus`][`crate::RentStatus`]
    fn rent_status<T: Environment>(
        &mut self,
        at_refcount: Option<core::num::NonZeroU32>,
    ) -> Result<RentStatus<T>>;

    /// Returns the current block number.
    ///
    /// # Note
    ///
    /// For more details visit: [`block_number`][`crate::block_number`]
    fn block_number<T: Environment>(&mut self) -> T::BlockNumber;

    /// Returns the minimum balance that is required for creating an account.
    ///
    /// # Note
    ///
    /// For more details visit: [`minimum_balance`][`crate::minimum_balance`]
    fn minimum_balance<T: Environment>(&mut self) -> T::Balance;

    /// Returns the tombstone deposit of the contract chain.
    ///
    /// # Note
    ///
    /// For more details visit: [`tombstone_deposit`][`crate::tombstone_deposit`]
    fn tombstone_deposit<T: Environment>(&mut self) -> T::Balance;

    /// Emits an event with the given event data.
    ///
    /// # Note
    ///
    /// For more details visit: [`emit_event`][`crate::emit_event`]
    fn emit_event<T, Event>(&mut self, event: Event)
    where
        T: Environment,
        Event: Topics + scale::Encode;

    /// Sets the rent allowance of the executed contract to the new value.
    ///
    /// # Note
    ///
    /// For more details visit: [`set_rent_allowance`][`crate::set_rent_allowance`]
    fn set_rent_allowance<T>(&mut self, new_value: T::Balance)
    where
        T: Environment;

    /// Invokes a contract message.
    ///
    /// # Note
    ///
    /// For more details visit: [`invoke_contract`][`crate::invoke_contract`]
    fn invoke_contract<T, Args>(
        &mut self,
        call_data: &CallParams<T, Args, ()>,
    ) -> Result<()>
    where
        T: Environment,
        Args: scale::Encode;

    /// Evaluates a contract message and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit: [`eval_contract`][`crate::eval_contract`]
    fn eval_contract<T, Args, R>(
        &mut self,
        call_data: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        T: Environment,
        Args: scale::Encode,
        R: scale::Decode;

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`instantiate_contract`][`crate::instantiate_contract`]
    fn instantiate_contract<T, Args, Salt, C>(
        &mut self,
        params: &CreateParams<T, Args, Salt, C>,
    ) -> Result<T::AccountId>
    where
        T: Environment,
        Args: scale::Encode,
        Salt: AsRef<[u8]>;

    /// Restores a smart contract tombstone.
    ///
    /// # Note
    ///
    /// For more details visit: [`restore_contract`][`crate::restore_contract`]
    fn restore_contract<T>(
        &mut self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) where
        T: Environment;

    /// Terminates a smart contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`terminate_contract`][`crate::terminate_contract`]
    fn terminate_contract<T>(&mut self, beneficiary: T::AccountId) -> !
    where
        T: Environment;

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Note
    ///
    /// For more details visit: [`transfer`][`crate::transfer`]
    fn transfer<T>(&mut self, destination: T::AccountId, value: T::Balance) -> Result<()>
    where
        T: Environment;

    /// Returns a random hash seed.
    ///
    /// # Note
    ///
    /// For more details visit: [`random`][`crate::random`]
    fn random<T>(&mut self, subject: &[u8]) -> Result<(T::Hash, T::BlockNumber)>
    where
        T: Environment;
}
