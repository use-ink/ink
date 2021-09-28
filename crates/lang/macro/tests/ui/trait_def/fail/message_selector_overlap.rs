use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, selector = 1)]
    fn message_1(&self);

    #[ink(message, selector = 1)]
    fn message_2(&self);
}

fn main() {}
