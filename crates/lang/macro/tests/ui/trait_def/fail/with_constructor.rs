use ink_lang as ink;

#[ink::trait_definition]
pub trait WithConstructor {
    #[ink(constructor)]
    fn new() -> Self;
}

fn main() {}
