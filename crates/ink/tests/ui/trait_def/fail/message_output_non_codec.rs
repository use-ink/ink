pub struct NonCodec;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self) -> NonCodec;
}

fn main() {}
