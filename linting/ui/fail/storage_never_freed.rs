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
        vec_field_1: Vec<AccountId>,
        vec_field_subscription: Vec<AccountId>,
        map_field_1: Mapping<AccountId, AccountId>,
        map_field_2: Mapping<AccountId, AccountId>,
        map_field_alias: MapAlias2<AccountId, AccountId>,
    }

    impl StorageNeverFreed {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                vec_field_1: Vec::new(),
                vec_field_subscription: Vec::new(),
                map_field_1: Mapping::new(),
                map_field_2: Mapping::new(),
                map_field_alias: Mapping::new(),
            }
        }

        fn flip(a: bool) -> bool {
            !a
        }

        #[ink(message)]
        pub fn add_to_fields(&mut self, v: AccountId) {
            self.vec_field_1.push(v);
            self.vec_field_subscription[0] = v;
            self.map_field_1.insert(v, &v);
            let _ = Self::flip(self.map_field_2.insert(v, &v).is_some());
            self.map_field_alias.insert(v, &v);
        }
    }
}

fn main() {}
