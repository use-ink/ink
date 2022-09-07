use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    unsafe fn message_ref(&self);

    #[ink(message)]
    unsafe fn message_mut(&mut self);
}

fn main() {}
