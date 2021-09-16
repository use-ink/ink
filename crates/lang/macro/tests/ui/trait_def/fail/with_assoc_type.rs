use ink_lang as ink;

#[ink::trait_definition]
pub trait WithAssocType {
    type InvalidAssocType;

    #[ink(message)]
    fn message(&self);
}

fn main() {}
