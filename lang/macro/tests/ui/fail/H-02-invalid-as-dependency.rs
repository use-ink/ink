use ink_lang as ink;

#[ink::contract(
    version = "0.1.0",
    compile_as_dependency = "yes",
)]
mod invalid_as_dependency {
    #[ink(storage)]
    struct InvalidAsDependency {}

    impl InvalidAsDependency {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
