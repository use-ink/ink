use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self);
    #[ink(message)]
    fn message_mut(&mut self);
}

fn main() {}
