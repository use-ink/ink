use ink_lang as ink;

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
        pub fn message_0_ref(&self) {}

        #[ink(message)]
        pub fn message_0_mut(&mut self) {}

        #[ink(message)]
        pub fn message_1_ref(&self) -> i8 {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_1_mut(&mut self) -> i8 {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_2_ref(&self) -> (i8, i16) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_2_mut(&mut self) -> (i8, i16) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_3_ref(&self) -> (i8, i16, i32) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_3_mut(&mut self) -> (i8, i16, i32) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_4_ref(&self) -> (i8, i16, i32, i64) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_4_mut(&mut self) -> (i8, i16, i32, i64) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_5_ref(&self) -> (i8, i16, i32, i64, i128) {
            unimplemented!()
        }

        #[ink(message)]
        pub fn message_5_mut(&mut self) -> (i8, i16, i32, i64, i128) {
            unimplemented!()
        }
    }
}

fn main() {}
