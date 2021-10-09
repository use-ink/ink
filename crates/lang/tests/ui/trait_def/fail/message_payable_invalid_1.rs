use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, payable = false)]
    fn message(&self);
}

fn main() {}
