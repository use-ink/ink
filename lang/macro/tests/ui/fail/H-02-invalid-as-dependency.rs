use ink_lang as ink;

#[ink::contract(compile_as_dependency = "yes")]
mod invalid_as_dependency {
    #[ink(storage)]
    pub struct InvalidAsDependency {}

    impl InvalidAsDependency {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
