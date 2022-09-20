#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message0(&self) {}

        #[ink(message)]
        pub fn message1(&self, _arg1: u8) {}

        #[ink(message)]
        pub fn message2(&self, _arg1: u8, _arg2: (u8, AccountId)) {}

        fn check_compiles(&self) {
            ink::env::pay_with_call!(self.message0(), 0);
            ink::env::pay_with_call!(self.message1(0), 0);
            ink::env::pay_with_call!(self.message2(0, (0, Self::env().account_id())), 0);
        }
    }
}

fn main() {}
