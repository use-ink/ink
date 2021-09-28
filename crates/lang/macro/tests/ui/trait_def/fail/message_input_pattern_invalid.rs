use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self, (a, b): (i32, i32));
}

fn main() {}
