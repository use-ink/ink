#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[derive(ink::Event)]
pub struct Flipped {
    pub flipped: bool,
}
