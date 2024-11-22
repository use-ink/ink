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

//! The public raw interface towards the host Wasm engine.

use crate::{
    backend::{
        EnvBackend,
        TypedEnvBackend,
    },
    call::{
        Call,
        CallParams,
        // CallV1,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAccountId,
        LimitParamsV1,
        LimitParamsV2,
    },
    engine::{
        EnvInstance,
        OnInstance,
    },
    event::Event,
    hash::{
        CryptoHash,
        HashOutput,
    },
    types::Gas,
    Environment,
    Result,
};
use ink_storage_traits::Storable;
// use pallet_revive_uapi::ReturnFlags;
use pallet_revive_uapi::ReturnFlags;

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
// pub fn gas_left<E>() -> Gas
// where
//     E: Environment,
// {
//     <EnvInstance as OnInstance>::on_instance(|instance| {
//         TypedEnvBackend::gas_left::<E>(instance)
//     })
// }

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
pub fn emit_event<E, Evt>(event: Evt)
where
    E: Environment,
    Evt: Event,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::emit_event::<E, Evt>(instance, event)
    })
}

/// Writes the value to the contract storage under the given storage key and returns the
/// size of pre-existing value if any.
///
/// # Panics
///
/// - If the encode length of value exceeds the configured maximum value length of a
///   storage entry.
pub fn set_contract_storage<K, V>(key: &K, value: &V) -> Option<u32>
where
    K: scale::Encode,
    V: Storable,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::set_contract_storage::<K, V>(instance, key, value)
    })
}

/// Returns the value stored under the given storage key in the contract's storage if any.
///
/// # Errors
///
/// - If the decoding of the typed value failed (`KeyNotFound`)
pub fn get_contract_storage<K, R>(key: &K) -> Result<Option<R>>
where
    K: scale::Encode,
    R: Storable,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::get_contract_storage::<K, R>(instance, key)
    })
}

/// Removes the `value` at `key`, returning the previous `value` at `key` from storage.
///
/// # Errors
///
/// - If the decoding of the typed value failed (`KeyNotFound`)
pub fn take_contract_storage<K, R>(key: &K) -> Result<Option<R>>
where
    K: scale::Encode,
    R: Storable,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::take_contract_storage::<K, R>(instance, key)
    })
}

/// Checks whether there is a value stored under the given storage key in the contract's
/// storage.
///
/// If a value is stored under the specified key, the size of the value is returned.
pub fn contains_contract_storage<K>(key: &K) -> Option<u32>
where
    K: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::contains_contract_storage::<K>(instance, key)
    })
}

/// Clears the contract's storage entry under the given storage key.
///
/// If a value was stored under the specified storage key, the size of the value is
/// returned.
pub fn clear_contract_storage<K>(key: &K) -> Option<u32>
where
    K: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        EnvBackend::clear_contract_storage::<K>(instance, key)
    })
}

/// Invokes a contract message and returns its result.
///
/// # Note
///
/// This is a low level way to evaluate another smart contract.
/// Prefer to use the ink! guided and type safe approach to using this.
///
/// **This will call into the original version of the host function. It is recommended to
/// use [`invoke_contract`] to use the latest version if the target runtime supports it.**
///
/// # Errors
///
/// - If the called account does not exist.
/// - If the called account is not a contract.
/// - If arguments passed to the called contract message are invalid.
/// - If the called contract execution has trapped.
/// - If the called contract ran out of gas upon execution.
/// - If the returned value failed to decode properly.
// pub fn invoke_contract_v1<E, Args, R>(
//     params: &CallParams<E, CallV1<E>, Args, R>,
// ) -> Result<ink_primitives::MessageResult<R>>
// where
//     E: Environment,
//     Args: scale::Encode,
//     R: scale::Decode,
// {
//     <EnvInstance as OnInstance>::on_instance(|instance| {
//         TypedEnvBackend::invoke_contract_v1::<E, Args, R>(instance, params)
//     })
// }

/// Invokes a contract message and returns its result.
///
/// # Note
///
/// **This will call into the latest version of the host function which allows setting new
/// weight and storage limit parameters.**
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
/// - If the called contract ran out of gas, proof size, or storage deposit upon
///   execution.
/// - If the returned value failed to decode properly.
pub fn invoke_contract<E, Args, R>(
    params: &CallParams<E, Call<E>, Args, R>,
) -> Result<ink_primitives::MessageResult<R>>
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
) -> Result<ink_primitives::MessageResult<R>>
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
/// This is a low level way to instantiate another smart contract, calling the latest
/// `instantiate_v2` host function.
///
/// Prefer to use methods on a `ContractRef` or the
/// [`CreateBuilder`](`crate::call::CreateBuilder`)
/// through [`build_create`](`crate::call::build_create`) instead.
///
/// # Errors
///
/// - If the code hash is invalid.
/// - If the arguments passed to the instantiation process are invalid.
/// - If the instantiation process traps.
/// - If the instantiation process runs out of gas.
/// - If given insufficient endowment.
/// - If the returned account ID failed to decode properly.
pub fn instantiate_contract<E, ContractRef, Args, Salt, R>(
    params: &CreateParams<E, ContractRef, LimitParamsV2<E>, Args, Salt, R>,
) -> Result<
    ink_primitives::ConstructorResult<<R as ConstructorReturnType<ContractRef>>::Output>,
>
where
    E: Environment,
    ContractRef: FromAccountId<E>,
    Args: scale::Encode,
    Salt: AsRef<[u8]>,
    R: ConstructorReturnType<ContractRef>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::instantiate_contract::<E, ContractRef, Args, Salt, R>(
            instance, params,
        )
    })
}

/// Instantiates another contract.
///
/// # Note
///
/// This is a low level way to instantiate another smart contract, calling the legacy
/// `instantiate_v1` host function.
///
/// Prefer to use methods on a `ContractRef` or the
/// [`CreateBuilder`](`crate::call::CreateBuilder`)
/// through [`build_create`](`crate::call::build_create`) instead.
///
/// # Errors
///
/// - If the code hash is invalid.
/// - If the arguments passed to the instantiation process are invalid.
/// - If the instantiation process traps.
/// - If the instantiation process runs out of gas.
/// - If given insufficient endowment.
/// - If the returned account ID failed to decode properly.
// pub fn instantiate_contract_v1<E, ContractRef, Args, Salt, R>(
//     params: &CreateParams<E, ContractRef, LimitParamsV1, Args, Salt, R>,
// ) -> Result<
//     ink_primitives::ConstructorResult<<R as ConstructorReturnType<ContractRef>>::Output>,
// >
// where
//     E: Environment,
//     ContractRef: FromAccountId<E>,
//     Args: scale::Encode,
//     Salt: AsRef<[u8]>,
//     R: ConstructorReturnType<ContractRef>,
// {
//     <EnvInstance as OnInstance>::on_instance(|instance| {
//         TypedEnvBackend::instantiate_contract_v1::<E, ContractRef, Args, Salt, R>(
//             instance, params,
//         )
//     })
// }

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
/// - If the transfer had brought the sender's total balance below the minimum balance.
///   You need to use [`terminate_contract`] in case this is your intention.
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
/// - The input is the 4-bytes selector followed by the arguments of the called function
///   in their SCALE encoded representation.
/// - No prior interaction with the environment must take place before calling this
///   procedure.
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
/// use ink_env::hash::{
///     HashOutput,
///     Sha2x256,
/// };
/// let input: &[u8] = &[13, 14, 15];
/// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
/// let hash = ink_env::hash_bytes::<Sha2x256>(input, &mut output);
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
///     243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248, 29, 136, 145, 115, 186,
///     134, 14, 175, 178, 99, 183, 21, 4, 94, 92, 69, 199, 207, 241, 179,
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
///     195, 218, 227, 165, 226, 17, 25, 160, 37, 92, 142, 238, 4, 41, 244, 211, 18, 94,
///     131, 116, 231, 116, 255, 164, 252, 248, 85, 233, 173, 225, 26, 185, 119, 235,
///     137, 35, 204, 251, 134, 131, 186, 215, 76, 112, 17, 192, 114, 243, 102, 166, 176,
///     140, 180, 124, 213, 102, 117, 212, 89, 89, 92, 209, 116, 17, 28,
/// ];
/// const message_hash: [u8; 32] = [
///     167, 124, 116, 195, 220, 156, 244, 20, 243, 69, 1, 98, 189, 205, 79, 108, 213,
///     78, 65, 65, 230, 30, 17, 37, 184, 220, 237, 135, 1, 209, 101, 229,
/// ];
/// const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
///     3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202, 127,
///     31, 13, 58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
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

/// Returns an Ethereum address from the ECDSA compressed public key.
///
/// # Example
///
/// ```
/// let pub_key = [
///     3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202, 127,
///     31, 13, 58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
/// ];
/// let EXPECTED_ETH_ADDRESS = [
///     253, 240, 181, 194, 143, 66, 163, 109, 18, 211, 78, 49, 177, 94, 159, 79, 207,
///     37, 21, 191,
/// ];
/// let mut output = [0; 20];
/// ink_env::ecdsa_to_eth_address(&pub_key, &mut output);
/// assert_eq!(output, EXPECTED_ETH_ADDRESS);
/// ```
///
/// # Errors
///
/// - If the ECDSA public key cannot be recovered from the provided public key.
pub fn ecdsa_to_eth_address(pubkey: &[u8; 33], output: &mut [u8; 20]) -> Result<()> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.ecdsa_to_eth_address(pubkey, output)
    })
}

/// Verifies a sr25519 signature.
///
/// # Example
///
/// ```
/// let signature: [u8; 64] = [
///     184, 49, 74, 238, 78, 165, 102, 252, 22, 92, 156, 176, 124, 118, 168, 116, 247,
///     99, 0, 94, 2, 45, 9, 170, 73, 222, 182, 74, 60, 32, 75, 64, 98, 174, 69, 55, 83,
///     85, 180, 98, 208, 75, 231, 57, 205, 62, 4, 105, 26, 136, 172, 17, 123, 99, 90,
///     255, 228, 54, 115, 63, 30, 207, 205, 131,
/// ];
/// let message: &[u8; 11] = b"hello world";
/// let pub_key: [u8; 32] = [
///     212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44,
///     133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
/// ];
///
/// let result = ink::env::sr25519_verify(&signature, message.as_slice(), &pub_key);
/// assert!(result.is_ok())
/// ```
///
/// # Errors
///
/// - If sr25519 signature cannot be verified.
///
/// **WARNING**: this function is from the [unstable interface](https://github.com/paritytech/substrate/tree/master/frame/contracts#unstable-interfaces),
/// which is unsafe and normally is not available on production chains.
pub fn sr25519_verify(
    signature: &[u8; 64],
    message: &[u8],
    pub_key: &[u8; 32],
) -> Result<()> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.sr25519_verify(signature, message, pub_key)
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

/// Retrieves the code hash of the contract at the specified account id.
///
/// # Errors
///
/// - If no code hash was found for the specified account id.
/// - If the returned value cannot be properly decoded.
pub fn code_hash<E>(account: &E::AccountId) -> Result<E::Hash>
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::code_hash::<E>(instance, account)
    })
}

/// Retrieves the code hash of the currently executing contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn own_code_hash<E>() -> Result<E::Hash>
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::own_code_hash::<E>(instance)
    })
}

/// Checks whether the caller of the current contract is the origin of the whole call
/// stack.
///
/// Prefer this over [`is_contract`] when checking whether your contract is being called
/// by a contract or a plain account. The reason is that it performs better since it does
/// not need to do any storage lookups.
///
/// A return value of `true` indicates that this contract is being called by a plain
/// account. and `false` indicates that the caller is another contract.
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

/// Replace the contract code at the specified address with new code.
///
/// # Note
///
/// There are a few important considerations which must be taken into account when
/// using this API:
///
/// 1. The storage at the code hash will remain untouched.
///
/// Contract developers **must ensure** that the storage layout of the new code is
/// compatible with that of the old code.
///
/// 2. The contract address (`AccountId`) remains the same, while the `code_hash` changes.
///
/// Contract addresses are initially derived from `hash(deploying_address ++ code_hash ++
/// salt)`. This makes it possible to determine a contracts address (`AccountId`) using
/// the `code_hash` of the *initial* code used to instantiate the contract.
///
/// However, because `set_code_hash` can modify the underlying `code_hash` of a contract,
/// it should not be relied upon that a contracts address can always be derived from its
/// stored `code_hash`.
///
/// 3. Re-entrant calls use new `code_hash`.
///
/// If a contract calls into itself after changing its code the new call would use the new
/// code. However, if the original caller panics after returning from the sub call it
/// would revert the changes made by `set_code_hash` and the next caller would use the old
/// code.
///
/// # Errors
///
/// `ReturnCode::CodeNotFound` in case the supplied `code_hash` cannot be found on-chain.
///
/// # Storage Compatibility
///
/// When the smart contract code is modified,
/// it is important to observe an additional virtual restriction
/// that is imposed on this procedure:
/// you should not change the order in which the contract state variables
/// are declared, nor their type.
///
/// Violating the restriction will not prevent a successful compilation,
/// but will result in the mix-up of values or failure to read the storage correctly.
/// This can result in severe errors in the application utilizing the contract.
///
/// If the storage of your contract looks like this:
///
/// ```ignore
/// #[ink(storage)]
/// pub struct YourContract {
///     x: u32,
///     y: bool,
/// }
/// ```
///
/// The procedures listed below will make it invalid:
///
/// Changing the order of variables:
///
/// ```ignore
/// #[ink(storage)]
/// pub struct YourContract {
///     y: bool,
///     x: u32,
/// }
/// ```
///
/// Removing existing variable:
///
/// ```ignore
/// #[ink(storage)]
/// pub struct YourContract {
///     x: u32,
/// }
/// ```
///
/// Changing type of a variable:
///
/// ```ignore
/// #[ink(storage)]
/// pub struct YourContract {
///     x: u64,
///     y: bool,
/// }
/// ```
///
/// Introducing a new variable before any of the existing ones:
///
/// ```ignore
/// #[ink(storage)]
/// pub struct YourContract {
///     z: Vec<u32>,
///     x: u32,
///     y: bool,
/// }
/// ```
///
/// Please refer to the
/// [Open Zeppelin docs](https://docs.openzeppelin.com/upgrades-plugins/1.x/writing-upgradeable#modifying-your-contracts)
/// for more details and examples.
pub fn set_code_hash<E>(code_hash: &E::Hash) -> Result<()>
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.set_code_hash(code_hash.as_ref())
    })
}

/// Tries to trigger a runtime dispatchable, i.e. an extrinsic from a pallet.
///
/// `call` (after SCALE encoding) should be decodable to a valid instance of `RuntimeCall`
/// enum.
///
/// For more details consult
/// [host function documentation](https://paritytech.github.io/substrate/master/pallet_contracts/api_doc/trait.Current.html#tymethod.call_runtime).
///
/// # Errors
///
/// - If the call cannot be properly decoded on the pallet contracts side.
/// - If the runtime doesn't allow for the contract unstable feature.
/// - If the runtime doesn't allow for dispatching this call from a contract.
///
/// # Panics
///
/// Panics in the off-chain environment.
pub fn call_runtime<E, Call>(call: &Call) -> Result<()>
where
    E: Environment,
    Call: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::call_runtime::<E, _>(instance, call)
    })
}

/// Adds a new delegate dependency lock to the contract.
///
/// This guarantees that the code of the dependency cannot be removed without first
/// calling [`unlock_delegate_dependency`]. It charges a fraction of the code
/// deposit, see [`pallet_contracts::Config::CodeHashLockupDepositPercent`](https://docs.rs/pallet-contracts/latest/pallet_contracts/pallet/trait.Config.html#associatedtype.CodeHashLockupDepositPercent) for details.
///
/// # Errors
///
/// - If the supplied `code_hash` cannot be found on-chain.
/// - If the `code_hash` is the same as the calling contract.
/// - If the maximum number of delegate dependencies is reached.
/// - If the delegate dependency already exists.
pub fn lock_delegate_dependency<E>(code_hash: &E::Hash)
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.lock_delegate_dependency::<E>(code_hash)
    })
}

/// Unlocks the delegate dependency from the contract.
///
/// This removes the lock and refunds the deposit from the call to
/// [`lock_delegate_dependency`]. The code of the dependency can be removed if the
/// reference count for the code hash is now zero.
///
/// # Errors
///
/// - If the delegate dependency does not exist.
pub fn unlock_delegate_dependency<E>(code_hash: &E::Hash)
where
    E: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.unlock_delegate_dependency::<E>(code_hash)
    })
}

/// Execute an XCM message locally, using the contract's address as the origin.
///
/// For more details consult the
/// [host function documentation](https://paritytech.github.io/substrate/master/pallet_contracts/api_doc/trait.Current.html#tymethod.xcm_execute).
///
/// # Errors
///
/// - If the message cannot be properly decoded on the `pallet-contracts` side.
/// - If the XCM execution fails because of the runtime's XCM configuration.
///
/// # Panics
///
/// Panics in the off-chain environment.
pub fn xcm_execute<E, Call>(msg: &xcm::VersionedXcm<Call>) -> Result<()>
where
    E: Environment,
    Call: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::xcm_execute::<E, _>(instance, msg)
    })
}

/// Send an XCM message, using the contract's address as the origin.
///
/// The `msg` argument has to be SCALE encoded, it needs to be decodable to a valid
/// instance of the `RuntimeCall` enum.
///
/// For more details consult
/// [host function documentation](https://paritytech.github.io/substrate/master/pallet_contracts/api_doc/trait.Current.html#tymethod.xcm_send).
///
/// # Errors
///
/// - If the message cannot be properly decoded on the `pallet-contracts` side.
///
/// # Panics
///
/// Panics in the off-chain environment.
pub fn xcm_send<E, Call>(
    dest: &xcm::VersionedLocation,
    msg: &xcm::VersionedXcm<Call>,
) -> Result<xcm::v4::XcmHash>
where
    E: Environment,
    Call: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        TypedEnvBackend::xcm_send::<E, _>(instance, dest, msg)
    })
}
