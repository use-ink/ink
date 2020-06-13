use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod forbidden_indents {
    #[ink(storage)]
    struct ForbiddenIndents {}

    impl ForbiddenIndents {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self) {
            // All identifiers starting with `__ink` are forbidden to use in ink!.
            let __ink_noop = ();
        }
    }
}

fn main() {}
