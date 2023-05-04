#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[derive(ink::Event, scale::Encode)]
#[cfg_attr(feature = "std", derive(ink::EventMetadata))]
pub struct Flipped {
    pub flipped: bool,
}
