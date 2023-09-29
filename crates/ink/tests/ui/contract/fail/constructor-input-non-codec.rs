#[ink::contract]
mod contract {
    #[::ink::scale_derive(TypeInfo)]
    pub struct NonCodecType;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor(_input: NonCodecType) -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
