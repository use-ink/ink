use ink_lang as ink;

// todo: add the wiring to detect the events metadata, and in cargo-contract merge together
// todo: consider the possibility of enforcing shared events to be an enum, and tied to an ink! trait def
// only a single event type per interface which encapsulates all the possible events
// inside an impl the emit_event is restricted to that trait impl type.
// in libs there is no such restriction, can just ink_env::emit_event?
// either way still need to do the metadata scanning
// also with cross-contract call even those events will not be included in the metadata.

#[ink::event_definition]
pub struct SharedEvent {
    arg_1: u8,
    #[ink(topic)]
    arg_2: u16,
}

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
            self.env().emit_event(super::SharedEvent { arg_1: 1, arg_2: 2 });
        }
    }
}

fn main() {}
