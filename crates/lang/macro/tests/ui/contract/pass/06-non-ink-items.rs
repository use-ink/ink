use ink_lang as ink;

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Noop {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn noop(&self) {}
    }

    struct NonInkStruct {}
    enum NonInkEnum {}
    impl NonInkStruct {
        fn do_nothing() {}
    }

    type NonInkTypeAlias = u32;
}

fn main() {}
