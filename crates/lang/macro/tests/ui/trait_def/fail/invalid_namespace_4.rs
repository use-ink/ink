use ink_lang as ink;

#[ink::trait_definition(namespace = true)]
pub trait InvalidNamespace {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
