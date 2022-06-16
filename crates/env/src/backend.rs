// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
        Call,
        CallParams,
        CreateParams,
        DelegateCall,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    topics::Topics,
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
    #[must_use]
    pub fn set_reverted(mut self, has_reverted: bool) -> Self {
        match has_reverted {
            true => self.value |= has_reverted as u32,
            false => self.value &= !has_reverted as u32,
        }
        self
    }

    /// Returns the underlying `u32` representation.
    #[cfg(not(feature = "std"))]
    pub(crate) fn into_u32(self) -> u32 {
        self.value
    }
}

/// The flags used to change the behavior of a contract call.
#[must_use]
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
    ///
    /// # Note
    ///
    /// This is an equivalent to the new [`set_contract_storage`][`Self::set_contract_storage`] method,
    /// but in order to maintain legacy behavior it returns nothing.
    fn set_contract_storage_compat<V>(&mut self, key: &Key, value: &V)
    where
        V: scale::Encode;

    /// Writes the value to the contract storage under the given key and returns
    /// the size of the pre-existing value at the specified key if any.
    fn set_contract_storage<V>(&mut self, key: &Key, value: &V) -> Option<u32>
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

    /// Returns the size of a value stored under the specified key is returned if any.
    fn contract_storage_contains(&mut self, key: &Key) -> Option<u32>;

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

    /// Retrieves an Ethereum address from the ECDSA compressed `pubkey`
    /// and stores the result in `output`.
    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
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

    /// Sets a new code hash for the current contract.
    ///
    /// This effectively replaces the code which is executed for this contract address.
    ///
    /// # Errors
    ///
    /// - If the supplied `code_hash` cannot be found on-chain.
    fn set_code_hash(&mut self, code_hash: &[u8]) -> Result<()>;
}

/// Environmental contract functionality.
pub trait TypedEnvBackend: EnvBackend {
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`caller`][`crate::caller`]
    fn caller<E: Environment>(&mut self) -> E::AccountId;

    /// Returns the transferred value for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`transferred_value`][`crate::transferred_value`]
    fn transferred_value<E: Environment>(&mut self) -> E::Balance;

    /// Returns the price for the specified amount of gas.
    ///
    /// # Note
    ///
    /// For more details visit: [`weight_to_fee`][`crate::weight_to_fee`]
    fn weight_to_fee<E: Environment>(&mut self, gas: u64) -> E::Balance;

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`gas_left`][`crate::gas_left`]
    fn gas_left<E: Environment>(&mut self) -> u64;

    /// Returns the timestamp of the current block.
    ///
    /// # Note
    ///
    /// For more details visit: [`block_timestamp`][`crate::block_timestamp`]
    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp;

    /// Returns the address of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`account_id`][`crate::account_id`]
    fn account_id<E: Environment>(&mut self) -> E::AccountId;

    /// Returns the balance of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`balance`][`crate::balance`]
    fn balance<E: Environment>(&mut self) -> E::Balance;

    /// Returns the current block number.
    ///
    /// # Note
    ///
    /// For more details visit: [`block_number`][`crate::block_number`]
    fn block_number<E: Environment>(&mut self) -> E::BlockNumber;

    /// Returns the minimum balance that is required for creating an account
    /// (i.e. the chain's existential deposit).
    ///
    /// # Note
    ///
    /// For more details visit: [`minimum_balance`][`crate::minimum_balance`]
    fn minimum_balance<E: Environment>(&mut self) -> E::Balance;

    /// Emits an event with the given event data.
    ///
    /// # Note
    ///
    /// For more details visit: [`emit_event`][`crate::emit_event`]
    fn emit_event<E, Event>(&mut self, event: Event)
    where
        E: Environment,
        Event: Topics + scale::Encode;

    /// Invokes a contract message and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit: [`invoke_contract`][`crate::invoke_contract`]
    fn invoke_contract<E, Args, R>(
        &mut self,
        call_data: &CallParams<E, Call<E>, Args, R>,
    ) -> Result<R>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode;

    /// Invokes a contract message via delegate call and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit: [`invoke_contract_delegate`][`crate::invoke_contract_delegate`]
    fn invoke_contract_delegate<E, Args, R>(
        &mut self,
        call_data: &CallParams<E, DelegateCall<E>, Args, R>,
    ) -> Result<R>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode;

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`instantiate_contract`][`crate::instantiate_contract`]
    fn instantiate_contract<E, Args, Salt, C>(
        &mut self,
        params: &CreateParams<E, Args, Salt, C>,
    ) -> Result<E::AccountId>
    where
        E: Environment,
        Args: scale::Encode,
        Salt: AsRef<[u8]>;

    /// Terminates a smart contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`terminate_contract`][`crate::terminate_contract`]
    fn terminate_contract<E>(&mut self, beneficiary: E::AccountId) -> !
    where
        E: Environment;

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Note
    ///
    /// For more details visit: [`transfer`][`crate::transfer`]
    fn transfer<E>(&mut self, destination: E::AccountId, value: E::Balance) -> Result<()>
    where
        E: Environment;

    /// Returns a random hash seed.
    ///
    /// # Note
    ///
    /// For more details visit: [`random`][`crate::random`]
    fn random<E>(&mut self, subject: &[u8]) -> Result<(E::Hash, E::BlockNumber)>
    where
        E: Environment;

    /// Checks whether a specified account belongs to a contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`is_contract`][`crate::is_contract`]
    #[allow(clippy::wrong_self_convention)]
    fn is_contract<E>(&mut self, account: &E::AccountId) -> bool
    where
        E: Environment;

    /// Checks whether the caller of the current contract is the origin of the whole call stack.
    ///
    /// # Note
    ///
    /// For more details visit: [`caller_is_origin`][`crate::caller_is_origin`]
    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment;

    /// Retrieves the code hash of the contract at the given `account` id.
    ///
    /// # Note
    ///
    /// For more details visit: [`code_hash`][`crate::code_hash`]
    fn code_hash<E>(&mut self, account: &E::AccountId) -> Result<E::Hash>
    where
        E: Environment;

    /// Retrieves the code hash of the currently executing contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`own_code_hash`][`crate::own_code_hash`]
    fn own_code_hash<E>(&mut self) -> Result<E::Hash>
    where
        E: Environment;
}
