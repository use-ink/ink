#![no_std]

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
    struct Flipper {
        /// The current state of our flag.
        value: storage::Value<bool>,
    }

    impl Deploy for Flipper {
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {
            self.value.set(false)
        }
    }

    impl Flipper {
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) {
            *self.value = !*self.value;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            println(&format!("Flipper Value: {:?}", *self.value));
            *self.value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Flipper;

    #[test]
    fn it_works() {
        let mut flipper = Flipper::deploy_mock();
        assert_eq!(flipper.get(), false);
        flipper.flip();
        assert_eq!(flipper.get(), true);
    }
}
