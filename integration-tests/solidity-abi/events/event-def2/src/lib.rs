#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::event]
pub struct EventDefAnotherCrate {
    #[ink(topic)]
    pub hash: [u8; 32],
}
