#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink::event_definition]
    pub enum Event {
        Event0 {},
        Event1 {
            #[ink(topic)]
            arg_1: i8,
        },
        Event2 {
            #[ink(topic)]
            arg_1: i8,
            #[ink(topic)]
            arg_2: i16,
        },
        Event3 {
            #[ink(topic)]
            arg_1: i8,
            #[ink(topic)]
            arg_2: i16,
            #[ink(topic)]
            arg_3: i32,
        },
        Event4 {
            #[ink(topic)]
            arg_1: i8,
            #[ink(topic)]
            arg_2: i16,
            #[ink(topic)]
            arg_3: i32,
            #[ink(topic)]
            arg_4: i64,
        },
        Event5 {
            #[ink(topic)]
            arg_1: i8,
            #[ink(topic)]
            arg_2: i16,
            #[ink(topic)]
            arg_3: i32,
            #[ink(topic)]
            arg_4: i64,
            // #[ink(topic)] <- Cannot have more than 4 topics by default.
            arg_5: i128,
        },
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self::env().emit_event(Event::Event0 {});
            Self::env().emit_event(Event::Event1 { arg_1: 1 });
            Self::env().emit_event(Event::Event2 { arg_1: 1, arg_2: 2 });
            Self::env().emit_event(Event::Event3 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
            });
            Self::env().emit_event(Event::Event4 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
            });
            Self::env().emit_event(Event::Event5 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
                arg_5: 5,
            });
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {
            self.env().emit_event(Event::Event0 {});
            self.env().emit_event(Event::Event1 { arg_1: 1 });
            self.env().emit_event(Event::Event2 { arg_1: 1, arg_2: 2 });
            self.env().emit_event(Event::Event3 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
            });
            self.env().emit_event(Event::Event4 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
            });
            self.env().emit_event(Event::Event5 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
                arg_5: 5,
            });
        }
    }
}

fn main() {}
