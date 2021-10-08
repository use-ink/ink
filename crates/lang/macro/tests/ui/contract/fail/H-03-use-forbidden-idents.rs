use ink_lang as ink;

#[ink::contract]
mod forbidden_indents {
    #[ink(storage)]
    pub struct ForbiddenIndents {}

    impl ForbiddenIndents {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        /// An ink! message starting with __ink_ prefix.
        #[ink(message)]
        pub fn __ink_message(&self) {
            // All identifiers starting with `__ink_` are forbidden to use in ink!.
            let __ink_first = ();
        }
    }
}

fn main() {}
