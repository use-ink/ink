#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[ink::chain_extension(extension = 13)]
pub trait Psp22Extension {
    type ErrorCode = Psp22Error;

    // PSP22 Metadata interfaces

    #[ink(function = 0x3d26)]
    fn token_name(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(function = 0x3420)]
    fn token_symbol(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(function = 0x7271)]
    fn token_decimals(asset_id: u32) -> Result<u8>;

    // PSP22 interface queries

    #[ink(function = 0x162d)]
    fn total_supply(asset_id: u32) -> Result<DefaultBalance>;

    #[ink(function = 0x6568)]
    fn balance_of(asset_id: u32, owner: DefaultAccountId) -> Result<DefaultBalance>;

    #[ink(function = 0x4d47)]
    fn allowance(
        asset_id: u32,
        owner: DefaultAccountId,
        spender: DefaultAccountId,
    ) -> Result<DefaultBalance>;

    // PSP22 transfer
    #[ink(function = 0xdb20)]
    fn transfer(asset_id: u32, to: DefaultAccountId, value: DefaultBalance)
        -> Result<()>;

    // PSP22 transfer_from
    #[ink(function = 0x54b3)]
    fn transfer_from(
        asset_id: u32,
        from: DefaultAccountId,
        to: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 approve
    #[ink(function = 0xb20f)]
    fn approve(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 increase_allowance
    #[ink(function = 0x96d6)]
    fn increase_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 decrease_allowance
    #[ink(function = 0xfecb)]
    fn decrease_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Psp22Error {
    TotalSupplyFailed,
}

pub type Result<T> = core::result::Result<T, Psp22Error>;

impl From<ink::scale::Error> for Psp22Error {
    fn from(_: ink::scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink::env::chain_extension::FromStatusCode for Psp22Error {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::TotalSupplyFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

/// An environment using default ink environment types, with PSP-22 extension included
#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(TypeInfo)]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = DefaultAccountId;
    type Balance = DefaultBalance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;

    type ChainExtension = crate::Psp22Extension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod psp22_ext {
    use super::{
        Result,
        Vec,
    };

    /// A chain extension which implements the PSP-22 fungible token standard.
    /// For more details see <https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md>
    #[ink(storage)]
    #[derive(Default)]
    pub struct Psp22Extension {}

    impl Psp22Extension {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        // PSP22 Metadata interfaces

        /// Returns the token name of the specified asset.
        #[ink(message, selector = 0x3d261bd4)]
        pub fn token_name(&self, asset_id: u32) -> Result<Vec<u8>> {
            self.env().extension().token_name(asset_id)
        }

        /// Returns the token symbol of the specified asset.
        #[ink(message, selector = 0x34205be5)]
        pub fn token_symbol(&self, asset_id: u32) -> Result<Vec<u8>> {
            self.env().extension().token_symbol(asset_id)
        }

        /// Returns the token decimals of the specified asset.
        #[ink(message, selector = 0x7271b782)]
        pub fn token_decimals(&self, asset_id: u32) -> Result<u8> {
            self.env().extension().token_decimals(asset_id)
        }

        // PSP22 interface queries

        /// Returns the total token supply of the specified asset.
        #[ink(message, selector = 0x162df8c2)]
        pub fn total_supply(&self, asset_id: u32) -> Result<Balance> {
            self.env().extension().total_supply(asset_id)
        }

        /// Returns the account balance for the specified asset & owner.
        #[ink(message, selector = 0x6568382f)]
        pub fn balance_of(&self, asset_id: u32, owner: AccountId) -> Result<Balance> {
            self.env().extension().balance_of(asset_id, owner)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`
        /// for the specified asset.
        #[ink(message, selector = 0x4d47d921)]
        pub fn allowance(
            &self,
            asset_id: u32,
            owner: AccountId,
            spender: AccountId,
        ) -> Result<Balance> {
            self.env().extension().allowance(asset_id, owner, spender)
        }

        // PSP22 transfer

        /// Transfers `value` amount of specified asset from the caller's account to the
        /// account `to`.
        #[ink(message, selector = 0xdb20f9f5)]
        pub fn transfer(
            &mut self,
            asset_id: u32,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env().extension().transfer(asset_id, to, value)
        }

        // PSP22 transfer_from

        /// Transfers `value` amount of specified asset on the behalf of `from` to the
        /// account `to`.
        #[ink(message, selector = 0x54b3c76e)]
        pub fn transfer_from(
            &mut self,
            asset_id: u32,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env()
                .extension()
                .transfer_from(asset_id, from, to, value)
        }

        // PSP22 approve

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount of the specified asset.
        #[ink(message, selector = 0xb20f1bbd)]
        pub fn approve(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env().extension().approve(asset_id, spender, value)
        }

        // PSP22 increase_allowance

        /// Atomically increases the allowance for the specified asset granted to
        /// `spender` by the caller.
        #[ink(message, selector = 0x96d6b57a)]
        pub fn increase_allowance(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env()
                .extension()
                .increase_allowance(asset_id, spender, value)
        }

        // PSP22 decrease_allowance

        /// Atomically decreases the allowance for the specified asset granted to
        /// `spender` by the caller.
        #[ink(message, selector = 0xfecb57d5)]
        pub fn decrease_allowance(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env()
                .extension()
                .decrease_allowance(asset_id, spender, value)
        }
    }
}
