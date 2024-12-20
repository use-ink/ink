#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::virtual_contract_ver2::VirtualContractVer2Ref;

#[ink::contract()]
mod virtual_contract_ver2 {

    #[ink(storage)]
    pub struct VirtualContractVer2 {
        version: [u8; 32],
        x: u32,
    }

    impl VirtualContractVer2 {
        /// Creates a new Template contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                version: [0; 32],
                x: 42
            }
        }

        #[ink(message)]
        pub fn set_x(&mut self, x: u32) {
            self.x = x - 1;
        }

        #[ink(message)]
        pub fn get_x(&self) -> u32 {
            self.x * 2
        }
    }

    impl Default for VirtualContractVer2 {
        fn default() -> Self {
            Self::new()
        }
    }
}
