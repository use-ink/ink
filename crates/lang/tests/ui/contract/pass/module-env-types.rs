use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {
        account_id: AccountId,
        balance: Balance,
        hash: Hash,
        timestamp: Timestamp,
        block_number: BlockNumber,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
