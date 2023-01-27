#![cfg_attr(not(feature = "std"), no_std)]

pub use self::fallback_contract::{
    FallbackContract,
    FallbackContractRef,
};

#[ink::contract]
mod fallback_contract {
    use main_contract::MainContractRef;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct FallbackContract {
        callee: MainContractRef,
    }

    impl FallbackContract {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(callee: MainContractRef) -> Self {
            Self { callee }
        }

        #[ink(message)]
        pub fn set_callee(&mut self, callee: MainContractRef) {
            self.callee = callee;
        }

        #[ink(message)]
        pub fn get_callee(&self) -> AccountId {
            self.callee.get_address()
        }

        #[ink(message)]
        pub fn get_address(&self) -> AccountId {
            self.env().account_id()
        }

        #[ink(message, selector = _)]
        pub fn fallback(&mut self) {
            self.callee.inc().unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn reentrancy_works() {
        use fallback_contract::{
            FallbackContract,
            FallbackContractRef,
        };
        use ink::primitives::Hash;
        use main_contract::{
            MainContract,
            MainContractRef,
        };

        let hash1 = Hash::from([10u8; 32]);
        let hash2 = Hash::from([20u8; 32]);

        ink::env::test::register_contract::<MainContract>(hash1.as_ref());
        ink::env::test::register_contract::<FallbackContract>(hash2.as_ref());

        let mut main_contract = MainContractRef::new()
            .code_hash(hash1)
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate();

        let fallback_contract = FallbackContractRef::new(main_contract.clone())
            .code_hash(hash2)
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate();

        let address1 = main_contract.get_address();
        let address2 = fallback_contract.get_address();

        main_contract.set_callee(address2);

        assert_eq!(main_contract.get_callee(), address2);
        assert_eq!(fallback_contract.get_callee(), address1);

        assert_eq!(main_contract.inc(), Ok(2));
        assert_eq!(main_contract.get(), 2);
    }
}
