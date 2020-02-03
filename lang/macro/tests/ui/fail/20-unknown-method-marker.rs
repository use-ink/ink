use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod noop {
    #[ink(storage)]
    struct Noop {}

    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn noop(&self) {}

        #[ink(unknown_marker)]
        fn has_unknown_marker(&self) {}

        #[ink(unknown_marker_2)]
        fn has_unknown_marker_too(&mut self) {}
    }
}

fn main() {}
