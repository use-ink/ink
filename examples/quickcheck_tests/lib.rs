#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod quickcheck_tests {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct QuickcheckTests {
        /// Stores a single `bool` value on the storage.
        value: i32,
    }

    impl QuickcheckTests {
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value += by;
        }

        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let quickcheck_tests = QuickcheckTests::new_default();
            assert_eq!(quickcheck_tests.get(), 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut quickcheck_tests = QuickcheckTests::new(0);
            assert_eq!(quickcheck_tests.get(), 0);
            quickcheck_tests.inc(1);
            assert_eq!(quickcheck_tests.get(), 1);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        // ink_e2e::quickcheck::quickcheck! {
        //     fn some(v: isize) -> bool {
        //         true
        //     }
        // }

        #[ink_e2e::test(quickcheck = true)]
        async fn it_works(mut client: ink_e2e::Client<C, E>, v: i32) -> E2EResult<()> {
            // given
            let constructor = QuickcheckTestsRef::new(0);
            let contract_acc_id = client
                .instantiate("quickcheck_tests", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<QuickcheckTestsRef>(contract_acc_id)
                .call(|quickcheck_tests| quickcheck_tests.get());
            let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_res.return_value(), 0));

            // when
            let inc = build_message::<QuickcheckTestsRef>(contract_acc_id.clone())
                .call(|quickcheck_tests| quickcheck_tests.inc(v));
            let _inc_res = client
                .call(&ink_e2e::bob(), inc, 0, None)
                .await
                .expect("flip failed");

            // then
            let get = build_message::<QuickcheckTestsRef>(contract_acc_id.clone())
                .call(|quickcheck_tests| quickcheck_tests.get());
            let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            //would fail the test with output containing: thread 'quickcheck_tests::e2e_tests::it_works' panicked at '[quickcheck] TEST FAILED (runtime error). Arguments: (0)
            //let v = v+1;
            //otherwise, the test passes in ~80s
            assert!(get_res.return_value() == v);

            Ok(())
        }

        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = QuickcheckTestsRef::new_default();

            // when
            let contract_acc_id = client
                .instantiate("quickcheck_tests", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // then
            let get = build_message::<QuickcheckTestsRef>(contract_acc_id.clone())
                .call(|quickcheck_tests| quickcheck_tests.get());
            let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_res.return_value(), 0));

            Ok(())
        }
    }
}
