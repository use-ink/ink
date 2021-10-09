use ink_lang as ink;

#[ink::trait_definition(namespace = true)]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
