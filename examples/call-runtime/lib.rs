#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod runtime_call {
    #[ink(storage)]
    pub struct Caller;

    impl Caller {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn make_transfer(&self) {
            self.env().call_runtime();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        #[should_panic(expected = "Cannot call runtime while off chain")]
        fn cannot_call_runtime_off_chain() {
            let mut contract = Caller::new();
            contract.make_transfer();
        }
    }
}
