use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    const fn message_ref(&self);

    #[ink(message)]
    const fn message_mut(&mut self);
}

fn main() {}
