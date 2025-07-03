#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {
        contract: Option<ContractRef>
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {
                contract: None
            }
        }

        #[ink(message)]
        pub fn message(&mut self, contract: ContractRef) {
            self.contract = Some(contract);
        }
    }
}

fn main() {}
