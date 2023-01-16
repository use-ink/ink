#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod complex_structures {
    use ink::storage::{
        traits::{
            AutoKey,
            ManualKey,
            Packed,
            StorageKey,
        },
        Lazy,
        Mapping,
    };

    #[ink::storage_item]
    struct NonPackedStruct {
        s1: Mapping<u32, u128>,
        s2: Lazy<u128>,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct PackedStruct {
        s1: u128,
        s2: Vec<u128>,
    }

    #[ink::storage_item(derive = false)]
    #[derive(Storable, StorableHint, StorageKey)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct NonPackedComplexStruct<KEY: StorageKey> {
        s1: (String, u128, PackedStruct),
        s2: Mapping<u128, u128>,
        s3: Lazy<u128>,
        s4: Mapping<u128, PackedStruct>,
        s5: Lazy<NonPackedStruct>,
        s6: PackedGeneric<PackedStruct>,
        s7: NonPackedGeneric<PackedStruct>,
    }

    #[ink(storage)]
    pub struct Contract {
        packed: PackedStruct,
        non_packed: NonPackedStruct,
        non_packed_complex_manual: NonPackedComplexStruct<ManualKey<123>>,
        non_packed_complex_auto: NonPackedComplexStruct<AutoKey>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }
    }
}
