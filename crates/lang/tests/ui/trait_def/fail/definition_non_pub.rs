use ink_lang as ink;

#[ink::trait_definition]
trait TraitDefinition {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
