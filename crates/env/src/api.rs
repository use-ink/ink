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

//! The public raw interface towards the host Wasm engine.

use crate::{
    backend::{
        EnvBackend,
        ReturnFlags,
        TypedEnvBackend,
    },
    call::{
        Call,
        CallParams,
        CreateParams,
        DelegateCall,
    },
    engine::{
        EnvInstance,
        OnInstance,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    topics::Topics,
    types::Gas,
    Environment,
    Result,
};
use ink_primitives::Key;

/// Returns the address of the caller of the executed contract.
///
/// # Errors
///
/// If the returned caller cannot be properly decoded.
pub fn caller<E>() -> E::AccountId
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::caller::<E>(instance)
    })
}

/// Returns the transferred value for the contract execution.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn transferred_value<E>() -> E::Balance
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::transferred_value::<E>(instance)
    })
}

/// Returns the price for the specified amount of gas.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn weight_to_fee<E>(gas: Gas) -> E::Balance
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::weight_to_fee::<E>(instance, gas)
    })
}

/// Returns the amount of gas left for the contract execution.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn gas_left<E>() -> Gas
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::gas_left::<E>(instance)
    })
}

/// Returns the current block timestamp.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn block_timestamp<E>() -> E::Timestamp
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::block_timestamp::<E>(instance)
    })
}

/// Returns the account ID of the executed contract.
///
/// # Note
///
/// This method was formerly known as `address`.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn account_id<E>() -> E::AccountId
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::account_id::<E>(instance)
    })
}

/// Returns the balance of the executed contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn balance<E>() -> E::Balance
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::balance::<E>(instance)
    })
}

/// Returns the current block number.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn block_number<E>() -> E::BlockNumber
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::block_number::<E>(instance)
    })
}

/// Returns the minimum balance that is required for creating an account
/// (i.e. the chain's existential deposit).
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn minimum_balance<E>() -> E::Balance
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::minimum_balance::<E>(instance)
    })
}

/// Emits an event with the given event data.
pub fn emit_event<E, Event>(event: Event)
where
    E: Environment,
    Event: Topics + scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::emit_event::<E, Event>(instance, event)
    })
}

/// Writes the value to the contract storage under the given key.
///
/// # Panics
///
/// - If the encode length of value exceeds the configured maximum value length of a storage entry.
pub fn set_contract_storage<V>(key: &Key, value: &V)
where
    V: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::set_contract_storage::<V>(instance, key, value)
    })
}

/// Returns the value stored under the given key in the contract's storage if any.
///
/// # Errors
///
/// - If the decoding of the typed value failed (`KeyNotFound`)
pub fn get_contract_storage<R>(key: &Key) -> Result<Option<R>>
where
    R: scale::Decode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::get_contract_storage::<R>(instance, key)
    })
}

/// Clears the contract's storage key entry.
pub fn clear_contract_storage(key: &Key) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::clear_contract_storage(instance, key)
    })
}

/// Invokes a contract message and returns its result.
///
/// # Note
///
/// This is a low level way to evaluate another smart contract.
/// Prefer to use the ink! guided and type safe approach to using this.
///
/// # Errors
///
/// - If the called account does not exist.
/// - If the called account is not a contract.
/// - If arguments passed to the called contract message are invalid.
/// - If the called contract execution has trapped.
/// - If the called contract ran out of gas upon execution.
/// - If the returned value failed to decode properly.
pub fn invoke_contract<E, Args, R>(params: &CallParams<E, Call<E>, Args, R>) -> Result<R>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::invoke_contract::<E, Args, R>(instance, params)
    })
}

/// Invokes a contract message via delegate call and returns its result.
///
/// # Note
///
/// This is a low level way to evaluate another smart contract via delegate call.
/// Prefer to use the ink! guided and type safe approach to using this.
///
/// # Errors
///
/// - If the specified code hash does not exist.
/// - If arguments passed to the called code message are invalid.
/// - If the called code execution has trapped.
pub fn invoke_contract_delegate<E, Args, R>(
    params: &CallParams<E, DelegateCall<E>, Args, R>,
) -> Result<R>
where
    E: Environment,
    Args: scale::Encode,
    R: scale::Decode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::invoke_contract_delegate::<E, Args, R>(instance, params)
    })
}

/// Instantiates another contract.
///
/// # Note
///
/// This is a low level way to instantiate another smart contract.
/// Prefer to use the ink! guided and type safe approach to using this.
///
/// # Errors
///
/// - If the code hash is invalid.
/// - If the arguments passed to the instantiation process are invalid.
/// - If the instantiation process traps.
/// - If the instantiation process runs out of gas.
/// - If given insufficient endowment.
/// - If the returned account ID failed to decode properly.
pub fn instantiate_contract<E, Args, Salt, C>(
    params: &CreateParams<E, Args, Salt, C>,
) -> Result<E::AccountId>
where
    E: Environment,
    Args: scale::Encode,
    Salt: AsRef<[u8]>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::instantiate_contract::<E, Args, Salt, C>(instance, params)
    })
}

/// Terminates the existence of the currently executed smart contract.
///
/// This removes the calling account and transfers all remaining balance
/// to the given beneficiary.
///
/// # Note
///
/// This function never returns. Either the termination was successful and the
/// execution of the destroyed contract is halted. Or it failed during the termination
/// which is considered fatal and results in a trap and rollback.
pub fn terminate_contract<E>(beneficiary: E::AccountId) -> !
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::terminate_contract::<E>(instance, beneficiary)
    })
}

/// Transfers value from the contract to the destination account ID.
///
/// # Note
///
/// This is more efficient and simpler than the alternative to make a no-op
/// contract call or invoke a runtime function that performs the
/// transaction.
///
/// # Errors
///
/// - If the contract does not have sufficient free funds.
/// - If the transfer had brought the sender's total balance below the
///   minimum balance. You need to use [`terminate_contract`] in case
///   this is your intention.
pub fn transfer<E>(destination: E::AccountId, value: E::Balance) -> Result<()>
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::transfer::<E>(instance, destination, value)
    })
}

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
pub fn decode_input<T>() -> Result<T>
where
    T: scale::Decode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::decode_input::<T>(instance)
    })
}

/// Returns the value back to the caller of the executed contract.
///
/// # Note
///
/// This function  stops the execution of the contract immediately.
pub fn return_value<R>(return_flags: ReturnFlags, return_value: &R) -> !
where
    R: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::return_value::<R>(instance, return_flags, return_value)
    })
}

/// Returns a random hash seed and the block number since which it was determinable
/// by chain observers.
///
/// # Note
///
/// - The subject buffer can be used to further randomize the hash.
/// - Within the same execution returns the same random hash for the same subject.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
///
/// # Important
///
/// The returned seed should only be used to distinguish commitments made before
/// the returned block number. If the block number is too early (i.e. commitments were
/// made afterwards), then ensure no further commitments may be made and repeatedly
/// call this on later blocks until the block number returned is later than the latest
/// commitment.
pub fn random<E>(subject: &[u8]) -> Result<(E::Hash, E::BlockNumber)>
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::random::<E>(instance, subject)
    })
}

/// Appends the given message to the debug message buffer.
pub fn debug_message(message: &str) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::debug_message(instance, message)
    })
}

/// Conducts the crypto hash of the given input and stores the result in `output`.
///
/// # Example
///
/// ```
/// use ink_env::hash::{Sha2x256, HashOutput};
/// let input: &[u8] = &[13, 14, 15];
/// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
/// let hash  = ink_env::hash_bytes::<Sha2x256>(input, &mut output);
/// ```
pub fn hash_bytes<H>(input: &[u8], output: &mut <H as HashOutput>::Type)
where
    H: CryptoHash,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.hash_bytes::<H>(input, output)
    })
}

/// Conducts the crypto hash of the given encoded input and stores the result in `output`.
///
/// # Example
///
/// ```
/// # use ink_env::hash::{Sha2x256, HashOutput};
/// const EXPECTED: [u8; 32] = [
///   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
///   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
/// ];
/// let encodable = (42, "foo", true); // Implements `scale::Encode`
/// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
/// ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);
/// assert_eq!(output, EXPECTED);
/// ```
pub fn hash_encoded<H, T>(input: &T, output: &mut <H as HashOutput>::Type)
where
    H: CryptoHash,
    T: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.hash_encoded::<H, T>(input, output)
    })
}

/// Recovers the compressed ECDSA public key for given `signature` and `message_hash`,
/// and stores the result in `output`.
///
/// # Example
///
/// ```
/// const signature: [u8; 65] = [
///     161, 234, 203,  74, 147, 96,  51, 212,   5, 174, 231,   9, 142,  48, 137, 201,
///     162, 118, 192,  67, 239, 16,  71, 216, 125,  86, 167, 139,  70,   7,  86, 241,
///      33,  87, 154, 251,  81, 29, 160,   4, 176, 239,  88, 211, 244, 232, 232,  52,
///     211, 234, 100, 115, 230, 47,  80,  44, 152, 166,  62,  50,   8,  13,  86, 175,
///      28,
/// ];
/// const message_hash: [u8; 32] = [
///     162, 28, 244, 179, 96, 76, 244, 178, 188,  83, 230, 248, 143, 106,  77, 117,
///     239, 95, 244, 171, 65, 95,  62, 153, 174, 166, 182,  28, 130,  73, 196, 208
/// ];
/// const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
///       2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160,  98, 149, 206, 135, 11,
///       7,   2, 155, 252, 219,  45, 206,  40, 217, 89, 242, 129,  91,  22, 248, 23,
///     152,
/// ];
/// let mut output = [0; 33];
/// ink_env::ecdsa_recover(&signature, &message_hash, &mut output);
/// assert_eq!(output, EXPECTED_COMPRESSED_PUBLIC_KEY);
/// ```
pub fn ecdsa_recover(
    signature: &[u8; 65],
    message_hash: &[u8; 32],
    output: &mut [u8; 33],
) -> Result<()> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.ecdsa_recover(signature, message_hash, output)
    })
}

/// Checks whether the specified account is a contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn is_contract<E>(account: &E::AccountId) -> bool
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::is_contract::<E>(instance, account)
    })
}

///
pub fn code_hash<E>(account: &E::AccountId, output: &mut E::Hash) -> Result<()>
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.code_hash::<E>(account, output)
    })
}

/// Checks whether the caller of the current contract is the origin of the whole call stack.
///
/// Prefer this over [`is_contract`] when checking whether your contract is being called by
/// a contract or a plain account. The reason is that it performs better since it does not
/// need to do any storage lookups.
///
/// A return value of `true` indicates that this contract is being called by a plain account.
/// and `false` indicates that the caller is another contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn caller_is_origin<E>() -> bool
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::caller_is_origin::<E>(instance)
    })
}
