use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message();
}

fn main() {}
