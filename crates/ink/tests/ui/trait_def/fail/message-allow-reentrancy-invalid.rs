#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, allow_reentrancy = false)]
    fn message(&self);
}

fn main() {}
