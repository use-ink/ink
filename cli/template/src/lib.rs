#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::{
    memory::format,
    storage,
};
use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// This simple dummy contract has a `bool` value that can
    /// alter between `true` and `false` using the `flip` message.
    /// Users can retrieve its current state using the `get` message.
    struct {{camel_name}} {
        /// The current state of our flag.
        value: storage::Value<bool>,
    }

    impl Deploy for {{camel_name}} {
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {
            self.value.set(false)
        }
    }

    impl {{camel_name}} {
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) {
            *self.value = !*self.value;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            env.println(&format!("Storage Value: {:?}", *self.value));
            *self.value
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut contract = {{camel_name}}::deploy_mock();
        assert_eq!(contract.get(), false);
        contract.flip();
        assert_eq!(contract.get(), true);
    }
}
