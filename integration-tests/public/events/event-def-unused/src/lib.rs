#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::trait_definition]
pub trait FlipperTrait {
    #[ink(message)]
    fn flip(&mut self);
}

#[ink::event]
pub struct EventDefUnused {
    #[ink(topic)]
    pub hash: [u8; 32],
    #[ink(topic)]
    pub maybe_hash: Option<[u8; 32]>,
}
