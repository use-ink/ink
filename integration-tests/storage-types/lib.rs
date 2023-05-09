//! # Integration Test for Storage Types
//!
//! This contract is made to showcase all of ink!'s storage types.
//! With this the proper decoding of the storage types can be tested.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod storage_types {
    use ink::prelude::{
        string::String,
        vec,
        vec::Vec,
    };
    use scale::{
        Decode,
        Encode,
    };

    #[derive(Debug, Decode, Encode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum CustomError {
        ErrorWithMessage(String),
    }

    #[derive(Clone, Debug, Decode, Default, Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum EnumWithoutValues {
        #[default]
        A,
        B,
        C,
    }

    #[derive(Clone, Debug, Decode, Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum EnumWithValues {
        OneValue(u32),
        TwoValues(u32, u32),
        ThreeValues(u32, u32, u32),
    }

    #[derive(Clone, Debug, Decode, Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct PrimitiveTypes {
        bool_value: bool,
        enum_without_values: EnumWithoutValues,
        enum_with_values: EnumWithValues,
        array_value: [u32; 3],
        tuple_value: (u32, u32),
    }

    #[derive(Clone, Debug, Decode, Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SignedIntegers {
        i128_value_max: i128,
        i128_value_min: i128,
        i16_value_max: i16,
        i16_value_min: i16,
        i32_value_max: i32,
        i32_value_min: i32,
        i64_value_max: i64,
        i64_value_min: i64,
        i8_value_max: i8,
        i8_value_min: i8,
    }

    #[derive(Clone, Debug, Decode, Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SubstrateTypes {
        account_id_value: AccountId,
        balance_value_max: Balance,
        balance_value_min: Balance,
        hash_value: Hash,
    }

    #[derive(Clone, Debug, Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct InkPreludeTypes {
        string_value: String,
        vec_string_value: Vec<String>,
        vec_vec_string_value: Vec<Vec<String>>,
    }

    #[derive(Clone, Decode, Encode)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct UnsignedIntegers {
        u128_value_max: u128,
        u128_value_min: u128,
        u16_value_max: u16,
        u16_value_min: u16,
        u32_value_max: u32,
        u32_value_min: u32,
        u64_value_max: u64,
        u64_value_min: u64,
        u8_value_max: u8,
        u8_value_min: u8,
    }

    #[ink(storage)]
    pub struct StorageTypes {
        ink_prelude_types: InkPreludeTypes,
        primitive_types: PrimitiveTypes,
        signed_integers: SignedIntegers,
        substrate_types: SubstrateTypes,
        unsigned_integers: UnsignedIntegers,
    }

    impl Default for StorageTypes {
        fn default() -> Self {
            Self::new()
        }
    }

    impl StorageTypes {
        #[ink(constructor)]
        pub fn new() -> Self {
            let vec_string_value = vec![
                String::from("This is a String"),
                String::from("This is another String"),
            ];
            let vec_vec_string_value = vec![vec_string_value.clone()];

            Self {
                unsigned_integers: UnsignedIntegers {
                    u128_value_max: u128::MAX,
                    u128_value_min: u128::MIN,
                    u16_value_max: u16::MAX,
                    u16_value_min: u16::MIN,
                    u32_value_max: u32::MAX,
                    u32_value_min: u32::MIN,
                    u64_value_max: u64::MAX,
                    u64_value_min: u64::MIN,
                    u8_value_max: u8::MAX,
                    u8_value_min: u8::MIN,
                },
                signed_integers: SignedIntegers {
                    i128_value_max: i128::MAX,
                    i128_value_min: i128::MIN,
                    i16_value_max: i16::MAX,
                    i16_value_min: i16::MIN,
                    i32_value_max: i32::MAX,
                    i32_value_min: i32::MIN,
                    i64_value_max: i64::MAX,
                    i64_value_min: i64::MIN,
                    i8_value_max: i8::MAX,
                    i8_value_min: i8::MIN,
                },
                ink_prelude_types: InkPreludeTypes {
                    string_value: String::from("This is a string"),
                    vec_string_value,
                    vec_vec_string_value,
                },
                primitive_types: PrimitiveTypes {
                    bool_value: true,
                    enum_with_values: EnumWithValues::ThreeValues(1, 2, 3),
                    enum_without_values: EnumWithoutValues::A,
                    array_value: [3, 2, 1],
                    tuple_value: (7, 8),
                },
                substrate_types: SubstrateTypes {
                    account_id_value: AccountId::from([0x00; 32]),
                    balance_value_max: Balance::MAX,
                    balance_value_min: Balance::MIN,
                    hash_value: Hash::from([0x00; 32]),
                },
            }
        }

        #[ink(message)]
        pub fn get_unsigned_integers(&self) -> UnsignedIntegers {
            self.unsigned_integers.clone()
        }

        #[ink(message)]
        pub fn get_signed_integers(&self) -> SignedIntegers {
            self.signed_integers.clone()
        }

        #[ink(message)]
        pub fn get_ink_prelude_types(&self) -> InkPreludeTypes {
            self.ink_prelude_types.clone()
        }

        #[ink(message)]
        pub fn get_substrate_types(&self) -> SubstrateTypes {
            self.substrate_types.clone()
        }

        #[ink(message)]
        pub fn get_primitive_types(&self) -> PrimitiveTypes {
            self.primitive_types.clone()
        }

        #[ink(message)]
        pub fn get_option_some(&self) -> Option<bool> {
            Some(true)
        }

        #[ink(message)]
        pub fn get_option_none(&self) -> Option<bool> {
            None
        }

        #[ink(message)]
        pub fn get_result_ok(&self) -> Result<bool, CustomError> {
            Ok(true)
        }

        #[ink(message)]
        pub fn get_result_error(&self) -> Result<bool, CustomError> {
            Err(CustomError::ErrorWithMessage(String::from(
                "This is the Error Message.",
            )))
        }

        #[ink(message)]
        pub fn get_panic(&self) -> Result<(), ()> {
            panic!("This is the Panic message.")
        }
    }

    #[cfg(test)]
    mod tests {}
}
