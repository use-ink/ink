use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod noop {
    #[ink(storage)]
    struct Noop {}

    #[ink(unknown_or_unsupported)]
    struct HasUnknownMarker {}

    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn noop(&self) {}
    }
}

fn main() {}
