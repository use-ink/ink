#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[derive(ink::Event, scale::Encode)]
pub struct Flipped {
    pub flipped: bool,
}
