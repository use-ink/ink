use ink_env::{
    DefaultEnvironment,
    Environment,
};
use ink_lang as ink;

pub struct EnvironmentMoreTopics;

impl ink_env::Environment for EnvironmentMoreTopics {
    const MAX_EVENT_TOPICS: usize = 10; // Default is 4.

    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type ChainExtension = ();
}

#[ink::contract(env = super::EnvironmentMoreTopics)]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink(event, anonymous)]
    pub struct EventWithManyTopics {
        #[ink(topic)]
        arg_1: i8,
        #[ink(topic)]
        arg_2: i16,
        #[ink(topic)]
        arg_3: i32,
        #[ink(topic)]
        arg_4: i64,
        #[ink(topic)]
        arg_5: i128,
        #[ink(topic)]
        arg_6: u8,
        #[ink(topic)]
        arg_7: u16,
        #[ink(topic)]
        arg_8: u32,
        #[ink(topic)]
        arg_9: u64,
        #[ink(topic)]
        arg_10: u128,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self::env().emit_event(EventWithManyTopics {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
                arg_5: 5,
                arg_6: 6,
                arg_7: 7,
                arg_8: 8,
                arg_9: 9,
                arg_10: 10,
            });
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {
            self.env().emit_event(EventWithManyTopics {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
                arg_4: 4,
                arg_5: 5,
                arg_6: 6,
                arg_7: 7,
                arg_8: 8,
                arg_9: 9,
                arg_10: 10,
            });
        }
    }
}

fn main() {}
