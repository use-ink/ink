use ink_lang as ink;

#[ink::interface]
pub mod contract_interface {
    #[ink(trait_definition)]
    pub trait ContractInterface {
        #[ink(constructor)]
        fn constructor() -> Self;

        #[ink(message)]
        fn message(&self, value: bool) {}
    }

    #[ink(event)]
    pub enum Event {
        MessageInvoked { value: bool }
    }
}

#[ink::contract]
mod contract {
    use super::contract_interface;

    #[ink(storage)]
    pub struct Contract {}

    impl contract_interface::ContractInterface for Contract {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self, value: bool) {
            self.env().emit_event(contract_interface::Event::MessageInvoked { value })
        }
    }
}

fn main() {}
