//! # Custom Allocator
//!
//! This example demonstrates how to opt-out of the ink! provided global memory allocator.
//!
//! We will use a custom bump allocator implementation as an example.
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
}

#[cfg(test)]
mod tests;