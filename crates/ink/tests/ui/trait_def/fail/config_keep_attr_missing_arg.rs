use ink_lang as ink;

#[ink::trait_definition(keep_attr)]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
