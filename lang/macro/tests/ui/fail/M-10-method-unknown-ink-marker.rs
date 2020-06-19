use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod unknown_method_ink_marker {
    #[ink(storage)]
    struct UnknownMethodInkMarker {}

    impl UnknownMethodInkMarker {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self) {}

        #[ink(unknown_marker)]
        fn method(&self) {}
    }
}

fn main() {}
