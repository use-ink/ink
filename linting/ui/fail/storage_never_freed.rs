#![cfg_attr(not(feature = "std"), no_main)]
#![cfg_attr(dylint_lib = "ink_linting", deny(storage_never_freed))]
pub type MapAlias1<K, V> = ink::storage::Mapping<K, V>;
pub type MapAlias2<K, V> = MapAlias1<K, V>;

#[ink::contract]
pub mod storage_never_freed {
    use crate::MapAlias2;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct StorageNeverFreed {
        // All the fields generate warnings, since there are `insert` operations for
        // them, but there are no `remove` operations.
        vec_1: Vec<AccountId>,
        vec_2: Vec<bool>,
        vec_subscription: Vec<AccountId>,
        map_1: Mapping<AccountId, AccountId>,
        map_2: Mapping<AccountId, AccountId>,
        map_3: Mapping<AccountId, AccountId>,
        map_alias: MapAlias2<AccountId, AccountId>,
    }

    impl StorageNeverFreed {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                vec_1: Vec::new(),
                vec_2: Vec::new(),
                vec_subscription: Vec::new(),
                map_1: Mapping::new(),
                map_2: Mapping::new(),
                map_3: Mapping::new(),
                map_alias: Mapping::new(),
            }
        }

        fn flip(a: bool) -> bool {
            !a
        }

        #[ink(message)]
        pub fn add_to_fields(&mut self, v: AccountId) {
            self.vec_1.push(v);
            self.vec_subscription[0] = v;
            self.map_1.insert(v, &v);
            let _ = Self::flip(self.map_2.insert(v, &v).is_some());
            self.map_alias.insert(v, &v);
            self.vec_2.push(self.map_3.insert(v, &v).is_some());
        }
    }
}

fn main() {}
