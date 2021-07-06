use ink_lang as ink;

#[ink::contract]
mod noop {

    struct Args {
        foo: ()
    }

    #[ink(storage)]
    pub struct Noop {}

    impl Noop {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn noop(&self, Foo { foo }: Foo) {}
    }
}

fn main() {}
