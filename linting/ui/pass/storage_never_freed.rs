#![cfg_attr(dylint_lib = "ink_linting", deny(storage_never_freed))]
pub type MapAlias<K, V> = ink::storage::Mapping<K, V>;

#[ink::contract]
pub mod storage_never_freed {
    use crate::MapAlias;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct StorageNeverFreed {
        vec_field: Vec<AccountId>,
        map_field: Mapping<AccountId, AccountId>,
        map_field2: MapAlias<AccountId, AccountId>,
        #[cfg_attr(dylint_lib = "ink_linting", allow(storage_never_freed))]
        map_field_suppressed: Mapping<AccountId, AccountId>,
    }

    impl StorageNeverFreed {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                vec_field: Vec::new(),
                map_field: Mapping::new(),
                map_field2: Mapping::new(),
                map_field_suppressed: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn add_to_fields(&mut self, v: AccountId) {
            self.vec_field.push(v);
            self.map_field.insert(v, &v);
            self.map_field2.insert(v, &v);
            self.map_field_suppressed.insert(v, &v);
        }

        #[ink(message)]
        pub fn remove_from_fields(&mut self, v: AccountId) {
            self.vec_field.pop();
            self.map_field.remove(v);
            self.map_field2.remove(v);
        }
    }
}

fn main() {}

