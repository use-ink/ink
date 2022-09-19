#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(constructor)]
    fn constructor() -> Self;
}

fn main() {}
