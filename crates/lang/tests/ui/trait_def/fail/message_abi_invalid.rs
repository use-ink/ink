use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    extern "C" fn message_ref(&self);

    #[ink(message)]
    extern "C" fn message_mut(&mut self);
}

fn main() {}
