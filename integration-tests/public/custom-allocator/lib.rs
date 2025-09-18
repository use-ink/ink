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
//! If you don't handle allocations correctly you can introduce security vulnerabilities
//! to your contracts.
//!
//! You may also introduce performance issues. This is because the code of your allocator
//! will be included in the final contract binary, potentially increasing gas usage
//! significantly.
//!
//! ## Why Change the Allocator?
//!
//! The default memory allocator was designed to have a tiny size footprint, and made some
//! compromises to achieve that, e.g it does not free/deallocate memory.
//!
//! You may have a use case where you want to deallocate memory, or allocate it using a
//! different strategy.
//!
//! Providing your own allocator lets you choose the right tradeoffs for your use case.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(sync_unsafe_cell)]
#![feature(allocator_api)]

// todo
// Here we set `dlmalloc` to be the global memory allocator.
//
// The [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) trait is
// important to understand if you're swapping our your allocator.
//#[cfg(not(feature = "std"))]
//#[global_allocator]
//static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

use core::{
    alloc::{
        GlobalAlloc,
        Layout,
    },
    cell::SyncUnsafeCell,
};

#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOC: BumpAllocator = BumpAllocator {};

pub struct BumpAllocator {}

struct BumpMemory {
    buffer: [u8; 1024 * 1024], // Pre-allocated memory buffer
    offset: usize,             // Current allocation offset
}

static mut MEMORY: Option<SyncUnsafeCell<BumpMemory>> = None;

#[allow(clippy::arithmetic_side_effects)]
unsafe impl GlobalAlloc for BumpAllocator {
    #[allow(static_mut_refs)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            if MEMORY.is_none() {
                MEMORY = Some(SyncUnsafeCell::new(BumpMemory {
                    buffer: [0; 1024 * 1024],
                    offset: 0,
                }));
            }
        }
        let memory = unsafe { &mut *MEMORY.as_ref().unwrap().get() };
        let start = memory.offset;
        let end = start + layout.size();

        if end > memory.buffer.len() {
            panic!("too large");
        } else {
            memory.offset = end;
            //ink::env::debug_println!("Allocated {} from {start} to {}", end-start,
            // end-1);
            &mut memory.buffer[start]
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op: deallocation is unsupported in a bump allocator.
    }
}

#[ink::contract]
mod custom_allocator {
    use ink::prelude::{
        vec,
        vec::Vec,
    };

    #[ink(storage)]
    pub struct CustomAllocator {
        /// Stores a single `bool` value on the storage.
        ///
        /// # Note
        ///
        /// We're using a `Vec` here as it allocates its elements onto the heap, as
        /// opposed to the stack. This allows us to demonstrate that our new
        /// allocator actually works.
        value: Vec<bool>,
    }

    impl CustomAllocator {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                value: vec![init_value],
            }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value[0] = !self.value[0];
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value[0]
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let custom_allocator = CustomAllocator::default();
            assert!(!custom_allocator.get());
        }

        #[ink::test]
        fn it_works() {
            let mut custom_allocator = CustomAllocator::new(false);
            assert!(!custom_allocator.get());
            custom_allocator.flip();
            assert!(custom_allocator.get());
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default
        /// constructor.
        #[ink_e2e::test]
        async fn default_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Given
            let mut constructor = CustomAllocatorRef::default();

            // When
            let contract = client
                .instantiate("custom_allocator", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<CustomAllocator>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(!get_result.return_value());

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // Given
            let mut constructor = CustomAllocatorRef::new(false);
            let contract = client
                .instantiate("custom_allocator", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CustomAllocator>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(!get_result.return_value());

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(get_result.return_value());

            Ok(())
        }
    }
}
