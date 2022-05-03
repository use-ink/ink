#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;
use ink_prelude::vec::Vec;

#[ink::chain_extension]
pub trait Psp22Extension {
    type ErrorCode = Psp22ErrorCode;

    // PSP22 Metadata interfaces

    #[ink(extension = 0x3d261bd4)]
    fn token_name(asset_id: u32) -> Result<Vec<u8>, Psp22Error>;

    #[ink(extension = 0x34205be5)]
    fn token_symbol(asset_id: u32) -> Result<Vec<u8>, Psp22Error>;

    #[ink(extension = 0x7271b782)]
    fn token_decimals(asset_id: u32) -> Result<u8, Psp22Error>;

    // PSP22 interface queries

    #[ink(extension = 0x162df8c2)]
    fn total_supply(
        asset_id: u32,
    ) -> Result<<ink_env::DefaultEnvironment as Environment>::Balance, Psp22Error>;

    #[ink(extension = 0x6568382f)]
    fn balance_of(
        asset_id: u32,
        owner: <ink_env::DefaultEnvironment as Environment>::AccountId,
    ) -> Result<<ink_env::DefaultEnvironment as Environment>::Balance, Psp22Error>;

    #[ink(extension = 0x4d47d921)]
    fn allowance(
        asset_id: u32,
        spender: <ink_env::DefaultEnvironment as Environment>::AccountId,
    ) -> Result<<ink_env::DefaultEnvironment as Environment>::Balance, Psp22Error>;

    // PSP22 transfer
    #[ink(extension = 0xdb20f9f5)]
    fn transfer(
        asset_id: u32,
        to: <ink_env::DefaultEnvironment as Environment>::AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), Psp22Error>;

    // PSP22 transfer_from
    #[ink(extension = 0x54b3c76e)]
    fn transfer_from(
        asset_id: u32,
        from: <ink_env::DefaultEnvironment as Environment>::AccountId,
        to: <ink_env::DefaultEnvironment as Environment>::AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), Psp22Error>;

    // PSP22 approve
    #[ink(extension = 0xb20f1bbd)]
    fn approve(
        asset_id: u32,
        spender: <ink_env::DefaultEnvironment as Environment>::AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), Psp22Error>;

    // PSP22 increase_allowance
    #[ink(extension = 0x96d6b57a)]
    fn increase_allowance(
        asset_id: u32,
        spender: <ink_env::DefaultEnvironment as Environment>::AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), Psp22Error>;

    // PSP22 decrease_allowance
    #[ink(extension = 0xfecb57d5)]
    fn decrease_allowance(
        asset_id: u32,
        spender: <ink_env::DefaultEnvironment as Environment>::AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), Psp22Error>;
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Psp22ErrorCode {
    TotalSupplyFailed,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Psp22Error {
    ErrorCode(Psp22ErrorCode),
}

impl From<Psp22ErrorCode> for Psp22Error {
    fn from(error_code: Psp22ErrorCode) -> Self {
        Self::ErrorCode(error_code)
    }
}

impl From<scale::Error> for Psp22Error {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink_env::chain_extension::FromStatusCode for Psp22ErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::TotalSupplyFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = crate::Psp22Extension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod psp22_ext_test {
    use super::Psp22Error;
    use super::Vec;

    #[ink(storage)]
    pub struct Psp22ExtTest {}

    impl Psp22ExtTest {
        #[ink(constructor)]
        pub fn construct() -> Self {
            Psp22ExtTest {}
        }

        // PSP22 Metadata interfaces

        /// Returns the token name of the specified asset.
        #[ink(message)]
        #[ink(selector = 0x3d261bd4)]
        pub fn token_name(&self, asset_id: u32) -> Result<Vec<u8>, Psp22Error> {
            self.env().extension().token_name(asset_id)
        }

        /// Returns the token symbol of the specified asset.
        #[ink(message)]
        #[ink(selector = 0x34205be5)]
        pub fn token_symbol(&self, asset_id: u32) -> Result<Vec<u8>, Psp22Error> {
            self.env().extension().token_symbol(asset_id)
        }

        /// Returns the token decimals of the specified asset.
        #[ink(message)]
        #[ink(selector = 0x7271b782)]
        pub fn token_decimals(&self, asset_id: u32) -> Result<u8, Psp22Error> {
            self.env().extension().token_decimals(asset_id)
        }

        // PSP22 interface queries

        /// Returns the total token supply of the specified asset.
        #[ink(message)]
        #[ink(selector = 0x162df8c2)]
        pub fn total_supply(&self, asset_id: u32) -> Result<Balance, Psp22Error> {
            self.env().extension().total_supply(asset_id)
        }

        /// Returns the account balance for the specified asset & owner.
        #[ink(message)]
        #[ink(selector = 0x6568382f)]
        pub fn balance_of(
            &self,
            asset_id: u32,
            owner: AccountId,
        ) -> Result<Balance, Psp22Error> {
            self.env().extension().balance_of(asset_id, owner)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner` for the specified asset.
        #[ink(message)]
        #[ink(selector = 0x4d47d921)]
        pub fn allowance(
            &self,
            asset_id: u32,
            spender: AccountId,
        ) -> Result<Balance, Psp22Error> {
            self.env().extension().allowance(asset_id, spender)
        }

        // PSP22 transfer

        /// Transfers `value` amount of specified asset from the caller's account to account `to`.
        #[ink(message)]
        #[ink(selector = 0xdb20f9f5)]
        pub fn transfer(
            &mut self,
            asset_id: u32,
            to: AccountId,
            value: Balance,
        ) -> Result<(), Psp22Error> {
            self.env().extension().transfer(asset_id, to, value)
        }

        // PSP22 transfer_from

        /// Transfers `value` amount of specified asset on the behalf of `from` to the account `to`.
        #[ink(message)]
        #[ink(selector = 0x54b3c76e)]
        pub fn transfer_from(
            &mut self,
            asset_id: u32,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(), Psp22Error> {
            self.env()
                .extension()
                .transfer_from(asset_id, from, to, value)
        }

        // PSP22 approve

        /// Allows `spender` to withdraw from the caller's account multiple times, up to the `value` amount of the specified asset.
        #[ink(message)]
        #[ink(selector = 0xb20f1bbd)]
        pub fn approve(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<(), Psp22Error> {
            self.env().extension().approve(asset_id, spender, value)
        }

        // PSP22 increase_allowance

        /// Atomically increases the allowance for the specified asset granted to `spender` by the caller.
        #[ink(message)]
        #[ink(selector = 0x96d6b57a)]
        pub fn increase_allowance(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<(), Psp22Error> {
            self.env()
                .extension()
                .increase_allowance(asset_id, spender, value)
        }

        // PSP22 decrease_allowance

        /// Atomically decreases the allowance for the specified asset granted to `spender` by the caller.
        #[ink(message)]
        #[ink(selector = 0xfecb57d5)]
        pub fn decrease_allowance(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<(), Psp22Error> {
            self.env()
                .extension()
                .decrease_allowance(asset_id, spender, value)
        }
    }
}
