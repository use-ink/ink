use ink_lang as ink;

#[ink::trait_definition]
pub unsafe trait TraitDefinition {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
