use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition<T> {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
