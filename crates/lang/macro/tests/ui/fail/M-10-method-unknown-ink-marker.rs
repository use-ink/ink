use ink_lang as ink;

#[ink::contract]
mod unknown_method_ink_marker {
    #[ink(storage)]
    pub struct UnknownMethodInkMarker {}

    impl UnknownMethodInkMarker {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}

        #[ink(unknown_marker)]
        pub fn method(&self) {}
    }
}

fn main() {}
