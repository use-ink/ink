#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, payable)]
    fn message(&self);
}

fn main() {}
