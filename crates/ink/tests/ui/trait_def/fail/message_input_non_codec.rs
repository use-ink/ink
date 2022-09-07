use ink_lang as ink;

pub struct NonCodec;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self, input: NonCodec);
}

fn main() {}
