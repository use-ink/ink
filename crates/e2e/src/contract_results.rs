use ink::codegen::ContractCallBuilder;
use ink_env::call::FromAccountId;
use ink_env::Environment;
use pallet_contracts_primitives::{CodeUploadResult, ContractInstantiateResult};
use std::fmt::Debug;

/// Result of a contract instantiation.
pub struct InstantiationResult<E: Environment, EventLog> {
    /// The account id at which the contract was instantiated.
    pub account_id: E::AccountId,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: ContractInstantiateResult<E::AccountId, E::Balance, ()>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

impl<E: Environment, EventLog> InstantiationResult<E, EventLog> {
    /// Returns the account id at which the contract was instantiated.
    pub fn call<Contract>(&self) -> <Contract as ContractCallBuilder>::Type
    where
        Contract: ContractCallBuilder,
        Contract::Type: FromAccountId<E>,
    {
        <<Contract as ContractCallBuilder>::Type as FromAccountId<E>>::from_account_id(
            self.account_id.clone(),
        )
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for `E`.
impl<E: Environment, EventLog> Debug for InstantiationResult<E, EventLog>
where
    E::AccountId: Debug,
    E::Balance: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InstantiationResult")
            .field("account_id", &self.account_id)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract upload.
pub struct UploadResult<E: Environment, EventLog> {
    /// The hash with which the contract can be instantiated.
    pub code_hash: E::Hash,
    /// The result of the dry run, contains debug messages if there were any.
    pub dry_run: CodeUploadResult<E::Hash, E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

/// We implement a custom `Debug` here, to avoid requiring the trait bound `Debug` for `E`.
impl<E: Environment, EventLog> Debug for UploadResult<E, EventLog>
where
    E::Balance: Debug,
    E::Hash: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("UploadResult")
            .field("code_hash", &self.code_hash)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}
