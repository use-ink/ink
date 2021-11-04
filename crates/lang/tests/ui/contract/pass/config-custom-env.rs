use ink_lang as ink;

pub struct CustomEnv;

impl ink_env::Environment for CustomEnv {
    const MAX_EVENT_TOPICS: usize = 3;
    type AccountId = [u8; 32];
    type Balance = u64;
    type Hash = [u8; 32];
    type Timestamp = u64;
    type BlockNumber = u64;
    type ChainExtension = ();
    // Too lazy to define a test type that fits the required trait bounds.
    type RentFraction =
        <ink_env::DefaultEnvironment as ink_env::Environment>::RentFraction;
}

#[ink::contract(env = super::CustomEnv)]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
