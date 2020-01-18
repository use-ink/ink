//! The public raw interface towards the host Wasm engine.
//!
//! # Note
//!
//! Prefer using the dedicated `EnvAccess` and `EnvAccessMut` types in order
//! to interoperate with the environment as they already have their associated
//! environemntal types associated to them and provide additional safety in some
//! scenarios.

use crate::{
    env3::{
        backend::{
            Env,
            TypedEnv,
        },
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        engine::{
            Accessor,
            Instance,
        },
        EnvTypes,
        Result,
        Topics,
    },
    storage::Key,
};

/// Returns the address of the caller of the executed contract.
///
/// # Errors
///
/// If the returned caller cannot be properly decoded.
pub fn caller<T>() -> Result<T::AccountId>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::caller::<T>(instance))
}

/// Returns the transferred balance for the contract execution.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn transferred_balance<T>() -> Result<T::Balance>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::transferred_balance::<T>(instance))
}

/// Returns the current price for gas.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn gas_price<T>() -> Result<T::Balance>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::gas_price::<T>(instance))
}

/// Returns the amount of gas left for the contract execution.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn gas_left<T>() -> Result<T::Balance>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::gas_left::<T>(instance))
}

/// Returns the current block time in milliseconds.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn now_in_ms<T>() -> Result<T::Moment>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::now_in_ms::<T>(instance))
}

/// Returns the address of the executed contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn address<T>() -> Result<T::AccountId>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::address::<T>(instance))
}

/// Returns the balance of the executed contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn balance<T>() -> Result<T::Balance>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::balance::<T>(instance))
}

/// Returns the current rent allowance for the executed contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn rent_allowance<T>() -> Result<T::Balance>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::rent_allowance::<T>(instance))
}

/// Returns the current block number.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn block_number<T>() -> Result<T::BlockNumber>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::block_number::<T>(instance))
}

/// Returns the minimum balance of the executed contract.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn minimum_balance<T>() -> Result<T::Balance>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| TypedEnv::minimum_balance::<T>(instance))
}

/// Emits an event with the given event data.
pub fn emit_event<T, Event>(event: Event)
where
    T: EnvTypes,
    Event: Topics<T> + scale::Encode,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::emit_event::<T, Event>(instance, event)
    })
}

/// Sets the rent allowance of the executed contract to the new value.
pub fn set_rent_allowance<T>(new_value: T::Balance)
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::set_rent_allowance::<T>(instance, new_value)
    })
}

/// Writes the value to the contract storage under the given key.
pub fn set_contract_storage<V>(key: Key, value: &V)
where
    V: scale::Encode,
{
    <Accessor as Instance>::run(|instance| {
        Env::set_contract_storage::<V>(instance, key, value)
    })
}

/// Returns the value stored under the given key in the contract's storage.
///
/// # Errors
///
/// - If the key's entry is empty
/// - If the decoding of the typed value failed
pub fn get_contract_storage<R>(key: Key) -> Result<R>
where
    R: scale::Decode,
{
    <Accessor as Instance>::run(|instance| Env::get_contract_storage::<R>(instance, key))
}

/// Clears the contract's storage key entry.
pub fn clear_contract_storage(key: Key) {
    <Accessor as Instance>::run(|instance| Env::clear_contract_storage(instance, key))
}

/// Invokes a contract message.
///
/// # Errors
///
/// - If the called contract does not exist.
/// - If the called contract is a tombstone.
/// - If arguments passed to the called contract message are invalid.
/// - If the called contract execution has trapped.
/// - If the called contract ran out of gas upon execution.
/// - If given too few endowment.
pub fn invoke_contract<T>(params: &CallParams<T, ()>) -> Result<()>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::invoke_contract::<T>(instance, params)
    })
}

/// Evaluates a contract message and returns its result.
///
/// # Errors
///
/// - If the called contract does not exist.
/// - If the called contract is a tombstone.
/// - If arguments passed to the called contract message are invalid.
/// - If the called contract execution has trapped.
/// - If the called contract ran out of gas upon execution.
/// - If given too few endowment.
/// - If the returned value failed to decode properly.
pub fn eval_contract<T, R>(params: &CallParams<T, ReturnType<R>>) -> Result<R>
where
    T: EnvTypes,
    R: scale::Decode,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::eval_contract::<T, R>(instance, params)
    })
}

/// Instantiates another contract.
///
/// # Errors
///
/// - If the code hash is invalid.
/// - If the arguments passed to the instantiation process are invalid.
/// - If the instantiation process traps.
/// - If the instantiation process runs out of gas.
/// - If given too few endowment.
/// - If the returned account ID failed to decode properly.
pub fn create_contract<T, C>(params: &CreateParams<T, C>) -> Result<T::AccountId>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::create_contract::<T, C>(instance, params)
    })
}

/// Restores a smart contract tombstone.
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
///
/// # Usage
///
/// A smart contract that has too few funds to pay for its storage fees
/// can eventually be evicted. An evicted smart contract `C` leaves behind
/// a tombstone associated with a hash that has been computed partially
/// by its storage contents.
///
/// To restore contract `C` back to a fully working contract the normal
/// process is to write another contract `C2` with the only purpose to
/// eventually have the absolutely same contract storage as `C` did when
/// it was evicted.
/// For that purpose `C2` can use other storage keys that have not been in
/// use by contract `C`.
/// Once `C2` contract storage matches the storage of `C` when it was evicted
/// `C2` can invoke this method in order to initiate restoration of `C`.
/// A tombstone hash is calculated for `C2` and if it matches the tombstone
/// hash of `C` the restoration is going to be successful.
/// The `filtered_keys` argument can be used to ignore the extraneous keys
/// used by `C2` but not used by `C`.
///
/// The process of such a smart contract restoration can generally be very expensive.
///
/// # Note
///
/// - The `filtered_keys` can be used to ignore certain storage regions
///   in the restorer contract to not influence the hash calculations.
/// - Does *not* perform restoration right away but defers it to the end of
///   the contract execution.
/// - Restoration is cancelled if there is no tombstone in the destination
///   address or if the hashes don't match. No changes are made in this case.
pub fn restore_contract<T>(
    account_id: T::AccountId,
    code_hash: T::Hash,
    rent_allowance: T::Balance,
    filtered_keys: &[Key],
) where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::restore_contract::<T>(
            instance,
            account_id,
            code_hash,
            rent_allowance,
            filtered_keys,
        )
    })
}

/// Returns the input to the executed contract.
///
/// # Note
///
/// - The input is the 4-bytes selector followed by the arguments
///   of the called function in their SCALE encoded representation.
/// - This property must be received as the first action an executed
///   contract to its environment and can only be queried once.
///   The environment access asserts this guarantee.
///
/// # Errors
///
/// - If the call to `input` is not the first call to the environment.
/// - If the input failed to decode into call data.
///     - This happens only if the host runtime provides less than 4 bytes for
///       the function selector upon this query.
pub fn input() -> Result<CallData> {
    <Accessor as Instance>::run(|instance| {
        Env::input(instance)
    })
}

/// Returns the value back to the caller of the executed contract.
///
/// # Note
///
/// This call must be the last call to the contract
/// environment for every contract execution.
pub fn output<R>(return_value: &R)
where
    R: scale::Encode,
{
    <Accessor as Instance>::run(|instance| {
        Env::output::<R>(instance, return_value)
    })
}

/// Returns a random hash.
///
/// # Note
///
/// The subject buffer can be used to further randomize the hash.
///
/// # Errors
///
/// If the returned value cannot be properly decoded.
pub fn random<T>(subject: &[u8]) -> Result<T::Hash>
where
    T: EnvTypes,
{
    <Accessor as Instance>::run(|instance| {
        TypedEnv::random::<T>(instance, subject)
    })
}

/// Prints the given contents to the environmental log.
pub fn println(content: &str) {
    <Accessor as Instance>::run(|instance| {
        Env::println(instance, content)
    })
}

/// Returns the value from the *runtime* storage at the position of the key.
///
/// # Errors
///
/// - If the key's entry is empty
/// - If the decoding of the typed value failed
pub fn get_runtime_storage<R>(runtime_key: &[u8]) -> Result<R>
where
    R: scale::Decode,
{
    <Accessor as Instance>::run(|instance| {
        Env::get_runtime_storage::<R>(instance, runtime_key)
    })
}
