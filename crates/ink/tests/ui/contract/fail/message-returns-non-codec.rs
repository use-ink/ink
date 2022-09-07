use ink_lang as ink;

#[ink::contract]
mod contract {
    #[derive(scale_info::TypeInfo)]
    pub struct NonCodecType;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) -> NonCodecType {
            NonCodecType
        }
    }
}

fn main() {}
