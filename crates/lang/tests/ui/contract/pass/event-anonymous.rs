use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink(event, anonymous)]
    pub struct Event0 {}

    #[ink(event, anonymous)]
    pub struct Event1 {
        #[ink(topic)]
        arg_1: i8,
    }

    #[ink(event, anonymous)]
    pub struct Event2 {
        #[ink(topic)]
        arg_1: i8,
        #[ink(topic)]
        arg_2: i16,
    }

    #[ink(event, anonymous)]
    pub struct Event3 {
        #[ink(topic)]
        arg_1: i8,
        #[ink(topic)]
        arg_2: i16,
        #[ink(topic)]
        arg_3: i32,
    }

    #[ink(event, anonymous)]
    pub struct Event4 {
        #[ink(topic)]
        arg_1: i8,
        #[ink(topic)]
        arg_2: i16,
        #[ink(topic)]
        arg_3: i32,
        #[ink(topic)]
        arg_4: i64,
    }

    #[ink(event, anonymous)]
    pub struct Event5 {
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
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self::env().emit_event(Event0 {});
            Self::env().emit_event(Event1 { arg_1: 1 });
            Self::env().emit_event(Event2 { arg_1: 1, arg_2: 2 });
            Self::env().emit_event(Event3 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
            });
            Self::env().emit_event(Event4 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
            });
            Self::env().emit_event(Event5 {
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
            self.env().emit_event(Event0 {});
            self.env().emit_event(Event1 { arg_1: 1 });
            self.env().emit_event(Event2 { arg_1: 1, arg_2: 2 });
            self.env().emit_event(Event3 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
            });
            self.env().emit_event(Event4 {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
            });
            self.env().emit_event(Event5 {
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
