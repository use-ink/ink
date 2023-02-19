//! # Custom Allocator
//!
//! This example demonstrates how to opt-out of the ink! provided global memory allocator.
//!
//! We will use [`dlmalloc`](https://github.com/alexcrichton/dlmalloc-rs) instead.
//!
//! ## Warning!
//!
//! We **do not** recommend you opt-out of the provided allocator for production contract
//! deployments!
//!
//! If you don't handle allocations correctly you can introduce security vulnerabilities to your
//! contracts.
//!
//! You may also introduce performance issues. This is because the code of your allocator will
//! be included in the final contract binary, potentially increasing gas usage significantly.
//!
//! ## Why Change the Allocator?
//!
//! The default memory allocator was designed to have a tiny size footprint, and made some
//! compromises to achieve that, e.g it does not free/deallocate memory
//!
//! You may have a use case where you want to deallocate memory, or allocate it using a different
//! strategy.
//!
//! Providing your own allocator lets you choose the right tradeoffs for your use case.

#![cfg_attr(not(feature = "std"), no_std)]
// Since we opted out of the default allocator we must also bring our own out-of-memory (OOM)
// handler. The Rust compiler doesn't let us do this unless we add this unstable/nightly feature.
#![cfg_attr(not(feature = "std"), feature(alloc_error_handler))]

// Here we set `dlmalloc` to be the global memory allocator.
//
// The [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) trait is
// important to understand if you're swapping our your allocator.
#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

// As mentioned earlier, we need to provide our own OOM handler.
//
// We don't try and handle this and opt to abort contract execution instead.
#[cfg(not(feature = "std"))]
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    core::arch::wasm32::unreachable()
}

#[ink::contract]
mod custom_allocator {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct CustomAllocator {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl CustomAllocator {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
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
            let custom_allocator = CustomAllocator::default();
            assert_eq!(custom_allocator.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut custom_allocator = CustomAllocator::new(false);
            assert_eq!(custom_allocator.get(), false);
            custom_allocator.flip();
            assert_eq!(custom_allocator.get(), true);
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = CustomAllocatorRef::default();

            // When
            let contract_account_id = client
                .instantiate("custom_allocator", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<CustomAllocatorRef>(contract_account_id.clone())
                .call(|custom_allocator| custom_allocator.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = CustomAllocatorRef::new(false);
            let contract_account_id = client
                .instantiate("custom_allocator", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<CustomAllocatorRef>(contract_account_id.clone())
                .call(|custom_allocator| custom_allocator.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<CustomAllocatorRef>(contract_account_id.clone())
                .call(|custom_allocator| custom_allocator.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<CustomAllocatorRef>(contract_account_id.clone())
                .call(|custom_allocator| custom_allocator.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
