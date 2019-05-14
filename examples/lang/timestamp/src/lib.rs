#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{
    env::println,
    memory::format,
    storage,
};
use ink_lang::contract;

contract! {
    /// This simple dummy contract has a `bool` value that can
    /// alter between `true` and `false` using the `flip` message.
    /// Users can retrieve its current state using the `get` message.
    struct Timestamp {
        /// The current state of our flag.
        value: storage::Value<bool>,
    }

    impl Deploy for Timestamp {
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {
            self.value.set(false)
        }
    }

    impl Timestamp {
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) {
            *self.value = !*self.value;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            println(&format!("Storage Value: {:?}", *self.value));
            *self.value
        }

        pub(external) fn get_now(&self) -> u64 {
            let now = env.now();
            println(&format!("Timestamp: {:?}", now));
            now
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut contract = Timestamp::deploy_mock();
        assert_eq!(contract.get(), false);
        contract.flip();
        assert_eq!(contract.get(), true);
        contract.get_now();
    }
}
