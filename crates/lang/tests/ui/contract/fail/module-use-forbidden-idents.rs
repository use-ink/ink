use ink_lang as ink;

#[ink::contract]
mod __ink_contract {
    #[ink(storage)]
    pub struct __ink_Contract {}

    const __ink_CONST: () = ();
    static __ink_STATIC: () = ();
    type __ink_Type = ();

    impl __ink_Contract {
        #[ink(constructor)]
        pub fn __ink_constructor(__ink_input: __ink_Type) -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn __ink_message(&self, __ink_input: __ink_Type) -> __ink_Type {
            let __ink_first = ();
        }
    }
}

fn main() {}
