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
            self.env().call_runtime().expect("Should succeed");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        #[should_panic(expected = "off-chain environment does not support `call runtime`")]
        fn cannot_call_runtime_off_chain() {
            let contract = Caller::new();
            contract.make_transfer();
        }
    }
}
