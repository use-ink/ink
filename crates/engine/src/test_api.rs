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

//! Operations on the off-chain testing environment.

use crate::{
    exec_context::{
        ExecContext,
        OffChainError,
    },
    ext,
    ext::ENV_INSTANCE,
    Environment,
};

/// Result of interacting with the environment.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can be encountered upon environmental interaction.
pub type Error = ink_env_types::Error<OffChainError>;

/// Record for an emitted event.
#[derive(Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<Vec<u8>>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

/// Sets the balance of the account to the given balance.
///
/// # Note
///
/// Note that account could refer to either a user account or
/// a smart contract account.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the underlying `new_balance` type does not match.
pub fn set_account_balance<T>(
    _account_id: T::AccountId,
    _new_balance: T::Balance,
) -> Result<()>
where
    T: Environment,
{
    unimplemented!("off-chain environment does not yet support `set_account_balance`");
}

/// Returns the balance of the account.
///
/// # Note
///
/// Note that account could refer to either a user account or
/// a smart contract account. This returns the same as `env::api::balance`
/// if given the account ID of the currently executed smart contract.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
pub fn get_account_balance<T>(_account_id: T::AccountId) -> Result<T::Balance>
where
    T: Environment,
{
    unimplemented!("off-chain environment does not yet support `get_account_balance`");
}

/// Sets the rent allowance of the contract account to the given rent allowance.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the underlying `new_rent_allowance` type does not match.
pub fn set_contract_rent_allowance<T>(
    _account_id: T::AccountId,
    _new_rent_allowance: T::Balance,
) -> Result<()>
where
    T: Environment,
{
    unimplemented!(
        "off-chain environment does not yet support `set_contract_rent_allowance`"
    );
}

/// Returns the rent allowance of the contract account.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the returned rent allowance cannot be properly decoded.
pub fn get_contract_rent_allowance<T>(_account_id: T::AccountId) -> Result<T::Balance>
where
    T: Environment,
{
    unimplemented!(
        "off-chain environment does not yet support `get_contract_rent_allowance`"
    );
}

/// Set the entropy hash of the current block.
///
/// # Note
///
/// This allows to control what [`random`][`crate::random`] returns.
pub fn set_block_entropy<T>(_entropy: T::Hash) -> Result<()>
where
    T: Environment,
{
    unimplemented!("off-chain environment does not yet support `set_block_entropy`");
}

/// Returns the contents of the past performed environmental `println` in order.
pub fn recorded_printlns() -> impl Iterator<Item = String> {
    // TODO
    vec![String::from("")].into_iter()
}

/// Advances the chain by a single block.
pub fn advance_block<T>() -> Result<()>
where
    T: Environment,
{
    unimplemented!("off-chain environment does not yet support `advance_block`");
}

/// Set to true to disable clearing storage
///
/// # Note
///
/// Useful for benchmarking because it ensures the initialized storage is maintained across runs,
/// because lazy storage structures automatically clear their associated cells when they are dropped.
pub fn set_clear_storage_disabled(_disable: bool) {
    unimplemented!(
        "off-chain environment does not yet support `set_clear_storage_disabled`"
    );
}

/// Initializes the whole off-chain environment.
///
/// # Note
///
/// - Initializes the off-chain environment with default values that fit most
/// uses cases.
pub fn initialize_or_reset_as_default<T>() -> Result<()>
where
    T: Environment,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        let alice: T::AccountId = default_accounts::<T>().expect("default").alice;
        instance.exec_context = Some(ExecContext {
            callee: scale::Encode::encode(&alice),
            caller: scale::Encode::encode(&alice),
        });
        instance.emitted_events = Vec::new();
        instance.count_reads = 0;
        instance.count_writes = 0;

        Ok(())
    })
}

/// Initializes the whole off-chain environment.
///
/// # Note
///
/// - Initializes the off-chain environment with default values that fit most
/// uses cases.
pub fn set_caller<T>(caller: T::AccountId)
where
    T: Environment,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        instance
            .exec_context
            .as_mut()
            .expect("uninitialized context")
            .caller = scale::Encode::encode(&caller);
    })
}
/// Returns the total number of reads and writes of the contract's storage.
pub fn get_contract_storage_rw<T>(_account_id: &T::AccountId) -> Result<(usize, usize)>
where
    T: Environment,
{
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        Ok((instance.count_reads, instance.count_writes))
    })
}

/// Returns the amount of storage cells used by the account `account_id`.
///
/// Returns `None` if the `account_id` is non-existent.
pub fn count_used_storage_cells<T>(_account_id: &T::AccountId) -> Result<usize>
where
    T: Environment,
{
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow_mut();
        Ok(instance.storage.len())
    })
}

/// Runs the given closure test function with the default configuartion
/// for the off-chain environment.
pub fn run_test<T, F>(f: F) -> Result<()>
where
    T: Environment,
    F: FnOnce(DefaultAccounts<T>) -> Result<()>,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    initialize_or_reset_as_default::<T>()?;
    let default_accounts = default_accounts::<T>()?;
    f(default_accounts)
}

/// Returns the recorded emitted events in order.
pub fn recorded_events() -> impl Iterator<Item = EmittedEvent> {
    ENV_INSTANCE.with(|instance| {
        let instance = &mut instance.borrow();
        instance.emitted_events.clone().into_iter()
    })
}

/// Returns the account id of the currently executing contract.
pub fn get_current_contract_account_id<T>() -> Result<T::AccountId>
where
    T: Environment,
{
    ENV_INSTANCE.with(|instance| {
        // TODO type hell
        let instance = &mut instance.borrow();
        let cont = instance.exec_context.as_ref().expect("no exec context");
        let res: std::result::Result<T::AccountId, scale::Error> =
            scale::Decode::decode(&mut &cont.callee[..]);
        let res: std::result::Result<T::AccountId, super::accounts::AccountError> =
            res.map_err(super::accounts::AccountError::from);
        let callee: std::result::Result<T::AccountId, OffChainError> =
            res.map_err(Into::into);
        let callee: std::result::Result<
            T::AccountId,
            ink_env_types::Error<OffChainError>,
        > = callee.map_err(|offchainerr| {
            ink_env_types::Error::<OffChainError>::OffChain(offchainerr)
        });
        callee
    })
}

/// Returns the default accounts for testing purposes:
/// Alice, Bob, Charlie, Django, Eve and Frank.
pub fn default_accounts<T>() -> Result<DefaultAccounts<T>>
where
    T: Environment,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    Ok(DefaultAccounts {
        alice: T::AccountId::from([0x01; 32]),
        bob: T::AccountId::from([0x02; 32]),
        charlie: T::AccountId::from([0x03; 32]),
        django: T::AccountId::from([0x04; 32]),
        eve: T::AccountId::from([0x05; 32]),
        frank: T::AccountId::from([0x06; 32]),
    })
}

/// The default accounts.
pub struct DefaultAccounts<T>
where
    T: Environment,
{
    /// The predefined `ALICE` account holding substantial amounts of value.
    pub alice: T::AccountId,
    /// The predefined `BOB` account holding some amounts of value.
    pub bob: T::AccountId,
    /// The predefined `CHARLIE` account holding some amounts of value.
    pub charlie: T::AccountId,
    /// The predefined `DJANGO` account holding no value.
    pub django: T::AccountId,
    /// The predefined `EVE` account holding no value.
    pub eve: T::AccountId,
    /// The predefined `FRANK` account holding no value.
    pub frank: T::AccountId,
}

impl From<ext::Error> for Error {
    fn from(ext_error: ext::Error) -> Self {
        match ext_error {
            ext::Error::UnknownError => Self::UnknownError,
            ext::Error::CalleeTrapped => Self::CalleeTrapped,
            ext::Error::CalleeReverted => Self::CalleeReverted,
            ext::Error::KeyNotFound => Self::KeyNotFound,
            ext::Error::BelowSubsistenceThreshold => Self::BelowSubsistenceThreshold,
            ext::Error::TransferFailed => Self::TransferFailed,
            ext::Error::NewContractNotFunded => Self::NewContractNotFunded,
            ext::Error::CodeNotFound => Self::CodeNotFound,
            ext::Error::NotCallable => Self::NotCallable,
        }
    }
}
