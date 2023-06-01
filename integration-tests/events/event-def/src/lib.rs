#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[derive(ink::Event, scale::Encode)]
#[cfg_attr(feature = "std", derive(ink::EventMetadata, scale::Decode))]
pub struct Flipped {
    pub value: bool,
}

#[derive(ink::Event, scale::Encode)]
#[cfg_attr(feature = "std", derive(ink::EventMetadata, scale::Decode))]
pub struct ThirtyTwoByteTopics {
    #[ink(topic)]
    pub hash: [u8; 32],
    #[ink(topic)]
    pub maybe_hash: Option<[u8; 32]>,
}
