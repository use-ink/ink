#[ink::trait_definition(namespace = "my_trait_namespace")]
pub trait WithNamespace {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
