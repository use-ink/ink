#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegatee2 {
    use ink::storage::{
        Mapping,
        traits::ManualKey,
    };
    #[ink(storage)]
    pub struct Delegatee2 {
        addresses: Mapping<Address, i32, ManualKey<0x23>>,
        counter: i32,
    }

    impl Delegatee2 {
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            /*
            unreachable!(
                "Constructors are not called when upgrading using `set_code_hash`."
            )
             */
            Self {
                addresses: Mapping::default(),
                counter: 0,
            }
        }

        /// Increments the current value.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.counter = self.counter.checked_add(3).unwrap();
        }

        /// Adds current value of counter to the `addresses`
        #[ink(message)]
        pub fn append_address_value(&mut self) {
            let caller = self.env().caller();
            self.addresses.insert(caller, &self.counter);
        }

        /// Increments the current value.
        /// todo
        #[ink(message)]
        pub fn code_hash(&self) -> ink::H256 {
            self.env()
                .code_hash(&self.env().address())
                .expect("no code hash could be found")
        }
    }
}
