#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegatee {
    use ink::storage::{
        traits::ManualKey,
        Mapping,
    };
    #[ink(storage)]
    pub struct Delegatee {
        addresses: Mapping<AccountId, i32, ManualKey<0x23>>,
        counter: i32,
        // Uncommenting below line will break storage compatibility.
        // flag: bool,
    }

    impl Delegatee {
        /// When using the delegate call. You only upload the code of the delegatee
        /// contract. However, the code and storage do not get initialized.
        ///
        /// Because of this. The constructor actually never gets called.
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            unreachable!(
                "Constructors are not called when upgrading using `set_code_hash`."
            )
        }

        /// Increments the current value.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.counter = self.counter.checked_add(2).unwrap();
        }

        /// Adds current value of counter to the `addresses`
        #[ink(message)]
        pub fn append_address_value(&mut self) {
            let caller = self.env().caller();
            self.addresses.insert(caller, &self.counter);
        }
    }
}
