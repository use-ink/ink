use ink_lang as ink;

#[ink::trait_definition]
pub trait WithAssocType {
    const INVALID_CONST: bool;

    #[ink(message)]
    fn message(&self);
}

fn main() {}
