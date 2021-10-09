use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message_ref<T>(&self);

    #[ink(message)]
    fn message_mut<T>(&mut self);
}

fn main() {}
