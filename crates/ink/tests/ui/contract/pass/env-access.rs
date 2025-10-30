#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let _ = Self::env().account_id();
            let _ = Self::env().balance();
            let _ = Self::env().block_timestamp();
            let _ = Self::env().block_number();
            let _ = Self::env().caller();
            let _ = Self::env().minimum_balance();
            let _ = Self::env().gas_limit();
            let _ = Self::env().gas_price();
            let _ = Self::env().ref_time_left();
            let _ = Self::env().call_data_size();
            let _ = Self::env().return_data_size();
            let _ = Self::env().transferred_value();
            let _ = Self::env().weight_to_fee(0);
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {
            let _ = self.env().account_id();
            let _ = self.env().balance();
            let _ = self.env().block_timestamp();
            let _ = self.env().block_number();
            let _ = self.env().caller();
            let _ = self.env().minimum_balance();
            let _ = self.env().gas_limit();
            let _ = self.env().gas_price();
            let _ = self.env().ref_time_left();
            let _ = self.env().call_data_size();
            let _ = self.env().return_data_size();
            let _ = self.env().transferred_value();
            let _ = self.env().weight_to_fee(0);
        }
    }
}

fn main() {}
