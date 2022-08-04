use ink_lang as ink;

#[ink::event_definition]
pub struct SharedEvent;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {
            self.env().emit_event(super::SharedEvent {});
        }
    }
}

fn main() {}
