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
        pub fn message_1_ref(&self, _input_1: i8) {}

        #[ink(message)]
        pub fn message_1_mut(&mut self, _input_1: i8) {}

        #[ink(message)]
        pub fn message_2_ref(&self, _input_1: i8, _input_2: i16) {}

        #[ink(message)]
        pub fn message_2_mut(&mut self, _input_1: i8, _input_2: i16) {}

        #[ink(message)]
        pub fn message_3_ref(&self, _input_1: i8, _input_2: i16, _input_3: i32) {}

        #[ink(message)]
        pub fn message_3_mut(&mut self, _input_1: i8, _input_2: i16, _input_3: i32) {}

        #[ink(message)]
        pub fn message_4_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
        ) {
        }

        #[ink(message)]
        pub fn message_4_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
        ) {
        }

        #[ink(message)]
        pub fn message_5_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
        ) {
        }

        #[ink(message)]
        pub fn message_5_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
        ) {
        }

        #[ink(message)]
        pub fn message_6_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
        ) {
        }

        #[ink(message)]
        pub fn message_6_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
        ) {
        }

        #[ink(message)]
        pub fn message_7_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
        ) {
        }

        #[ink(message)]
        pub fn message_7_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
        ) {
        }

        #[ink(message)]
        pub fn message_8_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
        ) {
        }

        #[ink(message)]
        pub fn message_8_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
        ) {
        }

        #[ink(message)]
        pub fn message_9_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
            _input_9: u64,
        ) {
        }

        #[ink(message)]
        pub fn message_9_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
            _input_9: u64,
        ) {
        }

        #[ink(message)]
        pub fn message_10_ref(
            &self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
            _input_9: u64,
            _input_10: u128,
        ) {
        }

        #[ink(message)]
        pub fn message_10_mut(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
            _input_9: u64,
            _input_10: u128,
        ) {
        }
    }
}

fn main() {}
