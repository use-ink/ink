use super::{
    super::TypedEncodedError,
    OffAccountId,
    OffBalance,
    OffBlockNumber,
    OffCall,
    OffHash,
    OffMoment,
};
use crate::{
    env3::EnvTypes,
    storage::Key,
};
use ink_prelude::collections::BTreeMap;
use derive_more::From;

/// Errors encountered upon interacting with the accounts database.
#[derive(Debug, From)]
pub enum AccountError {
    TypedEncoded(TypedEncodedError),
    #[from(ignore)]
    UnexpectedUserAccount,
}

/// Result type encountered while operating on accounts.
pub type Result<T> = core::result::Result<T, AccountError>;

/// The database that stores all accounts.
pub struct AccountsDb {
    /// The mapping from account ID to an actual account.
    accounts: BTreeMap<OffAccountId, Account>,
}

impl AccountsDb {
    /// Creates a new empty accounts database.
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }
}

/// An account within the chain.
pub struct Account {
    /// The balance of the account.
    balance: OffBalance,
    /// The kind of the account.
    kind: AccountKind,
}

impl Account {
    /// Returns the balance of the account.
    pub fn balance<T>(&self) -> Result<T::Balance>
    where
        T: EnvTypes,
    {
        self.balance.decode().map_err(Into::into)
    }

    /// Returns the contract account or an error if it is a user account.
    fn contract_or_err(&self) -> Result<&ContractAccount> {
        match &self.kind {
            AccountKind::User => Err(AccountError::UnexpectedUserAccount).map_err(Into::into),
            AccountKind::Contract(contract_account) => Ok(contract_account),
        }
    }

    /// Returns the contract account or an error if it is a user account.
    fn contract_or_err_mut(&mut self) -> Result<&mut ContractAccount> {
        match &mut self.kind {
            AccountKind::User => Err(AccountError::UnexpectedUserAccount).map_err(Into::into),
            AccountKind::Contract(contract_account) => Ok(contract_account),
        }
    }

    /// Returns the rent allowance of the contract account of an error.
    pub fn rent_allowance<T>(&self) -> Result<T::Balance>
    where
        T: EnvTypes,
    {
        self.contract_or_err().and_then(|contract| {
            contract.rent_allowance.decode().map_err(Into::into)
        })
    }

    /// Returns the code hash of the contract account of an error.
    pub fn code_hash<T>(&self) -> Result<T::Hash>
    where
        T: EnvTypes,
    {
        self.contract_or_err().and_then(|contract| {
            contract.code_hash.decode().map_err(Into::into)
        })
    }

    /// Sets the contract storage of key to the new value.
    pub fn set_storage<T>(&mut self, key: Key, new_value: &T) -> Result<()>
    where
        T: scale::Encode + 'static,
    {
        self.contract_or_err_mut().and_then(|contract| {
            todo!()
        })
    }

    /// Clears the contract storage at key.
    pub fn clear_storage(&mut self, key: Key) -> Result<()> {
        self.contract_or_err_mut().and_then(|contract| {
            todo!()
        })
    }

    /// Clears the contract storage at key.
    pub fn get_storage<T>(&self, key: Key) -> Result<T>
    where
        T: scale::Decode + 'static,
    {
        self.contract_or_err().and_then(|contract| {
            todo!()
        })
    }
}

/// The kind of the account.
///
/// Can be either a user account or a (more complicated) contract account.
pub enum AccountKind {
    User,
    Contract(ContractAccount),
}

/// Extraneous fields for contract accounts.
pub struct ContractAccount {
    /// The contract's rent allowance.
    rent_allowance: OffBalance,
    /// The contract's code hash.
    code_hash: OffHash,
    /// The contract storage.
    storage: ContractStorage,
}

/// The storage of a contract instance.
pub struct ContractStorage {
    /// The entries within the contract storage.
    entries: BTreeMap<Key, Vec<u8>>,
}
