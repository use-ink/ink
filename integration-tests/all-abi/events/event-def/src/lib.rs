#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::event]
pub struct ForeignFlipped {
    pub value: bool,
}

#[ink::event]
pub struct ThirtyTwoByteTopics {
    #[ink(topic)]
    pub hash: ink::sol::FixedBytes<32>,
    #[ink(topic)]
    pub maybe_hash: Option<ink::sol::FixedBytes<32>>,
}
