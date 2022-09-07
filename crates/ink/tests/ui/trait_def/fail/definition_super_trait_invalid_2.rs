use ink_lang as ink;

#[ink::trait_definition]
pub trait SuperTraitDefinition {
    #[ink(message)]
    fn super_message(&self);
}

#[ink::trait_definition]
pub trait TraitDefinition: SuperTraitDefinition {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
