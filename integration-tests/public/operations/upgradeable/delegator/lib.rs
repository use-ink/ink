#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegator {
    use ink::{
        env::{
            CallFlags,
            DefaultEnvironment,
            call::{
                ExecutionInput,
                Selector,
                build_call,
            },
        },
        primitives::H256,
        storage::{
            Lazy,
            Mapping,
            traits::ManualKey,
        },
    };

    #[ink(storage)]
    pub struct Delegator {
        addresses: Mapping<Address, i32, ManualKey<0x23>>,
        counter: i32,
        // TODO check if we even need to save the `H256` hash in here.
        delegate_to: Lazy<(H256, Address)>,
    }

    impl Delegator {
        /// Creates a new delegator smart contract with an initial value, and the hash of
        /// the contract code to delegate to.
        ///
        /// Additionally, this code hash will be locked to prevent its deletion, since
        /// this contract depends on it.
        #[ink(constructor)]
        pub fn new(init_value: i32, hash: H256, addr: Address) -> Self {
            let v = Mapping::new();

            let mut delegate_to = Lazy::new();
            delegate_to.set(&(hash, addr));

            Self {
                addresses: v,
                counter: init_value,
                delegate_to,
            }
        }

        /// Update the hash of the contract to delegate to.
        /// - Unlocks the old delegate dependency, releasing the deposit and allowing old
        ///   delegated to code to be removed.
        /// - Adds a new delegate dependency lock, ensuring that the new delegated to code
        ///   cannot be removed.
        #[ink(message)]
        pub fn update_delegate_to(&mut self, hash: H256, addr: Address) {
            if let Some(delegate_to) = self.delegate_to.get() {
                let _old_hash = delegate_to.0;
            }
            self.delegate_to.set(&(hash, addr));
        }

        /// Increment the current value using delegate call.
        #[ink(message)]
        pub fn inc_delegate(&mut self) {
            let selector = ink::selector_bytes!(Abi::Ink, "inc");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(self.delegate_to().1)
                // We specify `CallFlags::TAIL_CALL` to use the delegatee last memory frame
                // as the end of the execution cycle.
                // So any mutations to `Packed` types, made by delegatee,
                // will be flushed to storage.
                //
                // If we don't specify this flag.
                // The storage state before the delegate call will be flushed to storage instead.
                // See https://substrate.stackexchange.com/questions/3336/i-found-set-allow-reentry-may-have-some-problems/3352#3352
                .call_flags(CallFlags::TAIL_CALL)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
        }

        /// Adds entry to `addresses` using delegate call.
        /// Note that we don't need `CallFlags::TAIL_CALL` flag
        /// because `Mapping` updates the storage instantly on-demand.
        #[ink(message)]
        pub fn add_entry_delegate(&mut self) {
            let selector = ink::selector_bytes!(Abi::Ink, "append_address_value");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(self.delegate_to().1)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
        }

        /// Returns the current value of the counter.
        #[ink(message)]
        pub fn get_counter(&self) -> i32 {
            self.counter
        }

        /// Returns the current value of the address.
        #[ink(message)]
        pub fn get_value(&self, address: Address) -> (Address, Option<i32>) {
            (self.env().caller(), self.addresses.get(address))
        }

        fn delegate_to(&self) -> (H256, Address) {
            self.delegate_to
                .get()
                .expect("delegate_to always has a value")
        }
    }

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;

}
