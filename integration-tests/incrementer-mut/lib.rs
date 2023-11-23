#![cfg_attr(not(feature = "std"), no_std, no_main)]

//! A simple incrementer contract
//! demonstrating the internal mutability of message parameters.

pub use self::incrementer_mut::{
    Incrementer,
    IncrementerRef,
};

#[ink::contract]
mod incrementer_mut {
    #[ink(storage)]
    pub struct Incrementer {
        value: i32,
    }

    impl Incrementer {
        /// Create a new contract with the specified counter value.
        /// If it is below 0, it is set to 0
        #[ink(constructor)]
        pub fn new(mut init_value: i32) -> Self {
            if init_value < 0 {
                init_value = 0;
            }
            Self { value: init_value }
        }

        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value = self.value.checked_add(by).unwrap();
        }

        /// Update the counter with the specified value.
        /// If it is above 100, we set it to 0.
        #[ink(message)]
        pub fn update(&mut self, mut value: i32) {
            if value > 100 {
                value = 0;
            }
            self.value = value;
        }

        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let contract = Incrementer::new_default();
            assert_eq!(contract.get(), 0);
        }

        #[ink::test]
        fn it_works() {
            let mut contract = Incrementer::new(42);
            assert_eq!(contract.get(), 42);
            contract.inc(5);
            assert_eq!(contract.get(), 47);
            contract.inc(-50);
            assert_eq!(contract.get(), -3);
        }
        #[ink::test]
        fn mutability_works() {
            let mut contract = Incrementer::new(-5);
            assert_eq!(contract.get(), 0);
            contract.update(80);
            assert_eq!(contract.get(), 80);
            contract.update(120);
            assert_eq!(contract.get(), 0);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = IncrementerRef::new(-2);
            let contract = client
                .instantiate("incrementer_mut", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Incrementer>();

            let get = call.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), 0));

            // when
            let flip = call.update(50);
            let _flip_res = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("update failed");

            // then
            let get = call.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), 50));

            // when
            let flip = call.update(150);
            let _flip_res = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("update failed");

            // then
            let get = call.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), 0));

            Ok(())
        }
    }
}
