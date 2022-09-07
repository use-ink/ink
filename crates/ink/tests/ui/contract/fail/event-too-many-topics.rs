use ink_env::{
    DefaultEnvironment,
    Environment,
};
use ink_lang as ink;

pub struct EnvironmentMoreTopics;

impl ink_env::Environment for EnvironmentMoreTopics {
    const MAX_EVENT_TOPICS: usize = 2;

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

    #[ink(event)]
    pub struct Event {
        #[ink(topic)]
        arg_1: i8,
        #[ink(topic)]
        arg_2: i16,
        #[ink(topic)]
        arg_3: i32,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self::env().emit_event(Event {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
            });
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {
            self.env().emit_event(Event {
                arg_1: 1,
                arg_2: 2,
                arg_3: 3,
            });
        }
    }
}

fn main() {}
