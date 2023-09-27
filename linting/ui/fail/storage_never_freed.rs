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
        vec_field: Vec<AccountId>,
        map_field: Mapping<AccountId, AccountId>,
        map_field2: MapAlias2<AccountId, AccountId>,
    }

    impl StorageNeverFreed {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                vec_field: Vec::new(),
                map_field: Mapping::new(),
                map_field2: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn add_to_fields(&mut self, v: AccountId) {
            self.vec_field.push(v);
            self.map_field.insert(v, &v);
            self.map_field2.insert(v, &v);
        }
    }
}

fn main() {}
