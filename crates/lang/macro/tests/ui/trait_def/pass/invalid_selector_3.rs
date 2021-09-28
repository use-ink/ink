use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, selector = 0xC0DECAFE)]
    fn message(&self);
}

fn main() {}
