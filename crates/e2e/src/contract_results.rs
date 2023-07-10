use ink::codegen::ContractCallBuilder;
use ink_env::call::FromAccountId;
use ink_env::Environment;
use pallet_contracts_primitives::ContractInstantiateResult;
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
