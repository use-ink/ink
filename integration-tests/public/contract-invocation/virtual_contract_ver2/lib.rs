#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::virtual_contract_ver2::VirtualContractVer2Ref;

#[ink::contract]
mod virtual_contract_ver2 {

    #[ink(storage)]
    pub struct VirtualContractVer2 {
        version: Address,
        x: u32,
    }

    impl VirtualContractVer2 {
        /// Creates a new Template contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                version: Address::zero(),
                x: 7,
            }
        }

        #[ink(message)]
        pub fn set_x(&mut self, x: u32) {
            self.x = x.saturating_sub(1);
        }

        #[ink(message)]
        pub fn get_x(&self) -> u32 {
            self.x.saturating_mul(2)
        }
    }

    impl Default for VirtualContractVer2 {
        fn default() -> Self {
            Self::new()
        }
    }
}
