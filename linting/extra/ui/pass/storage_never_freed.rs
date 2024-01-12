#![cfg_attr(not(feature = "std"), no_main)]
#![cfg_attr(dylint_lib = "ink_linting", deny(storage_never_freed))]
pub type MapAlias<K, V> = ink::storage::Mapping<K, V>;

#[ink::contract]
pub mod storage_never_freed {
    use crate::MapAlias;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct StorageNeverFreed {
        vec_1: Vec<AccountId>,
        map_1: Mapping<AccountId, AccountId>,
        map_2: MapAlias<AccountId, AccountId>,
        #[cfg_attr(dylint_lib = "ink_linting", allow(storage_never_freed))]
        map_field_suppressed: Mapping<AccountId, AccountId>,

        // Vec which buffer was used unsafe operations with their raw pointers are not
        // reported
        vec_field_mut_pointer: Vec<AccountId>,
    }

    impl StorageNeverFreed {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                vec_1: Vec::new(),
                map_1: Mapping::new(),
                map_2: Mapping::new(),
                map_field_suppressed: Mapping::new(),
                vec_field_mut_pointer: Vec::new(),
            }
        }

        #[ink(message)]
        pub fn add_to_fields(&mut self, v: AccountId) {
            self.vec_1.push(v);
            self.map_1.insert(v, &v);
            self.map_2.insert(v, &v);
            self.map_field_suppressed.insert(v, &v);

            // Should not be reported, since elements may be removed using the pointer
            self.vec_field_mut_pointer[0] = v;
            unsafe {
                let ptr = self.vec_field_mut_pointer.as_mut_ptr();
                let new_len = self.vec_field_mut_pointer.len() - 1;
                std::ptr::copy(ptr.offset(1), ptr, new_len);
                self.vec_field_mut_pointer.set_len(new_len);
            }
        }

        #[ink(message)]
        pub fn remove_from_fields(&mut self, v: AccountId) {
            self.vec_1.pop();
            self.map_1.remove(v);
            self.map_2.remove(v);
        }
    }
}

fn main() {}
