#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    use ink::storage::traits::StorageLayout;

    #[ink(storage)]
    pub struct Contract {
        packed: PackedFields,
    }

    #[derive(Debug, Default)]
    #[cfg_attr(feature = "std", derive(StorageLayout))]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct PackedFields {
        field_1: i8,
        field_2: i16,
        field_3: i32,
        field_4: i64,
        field_5: i128,
        field_6: u8,
        field_7: u16,
        field_8: u32,
        field_9: u64,
        field_10: u128,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {
                packed: Default::default(),
            }
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
