use ink_lang as ink;

#[ink::trait_definition(namespace = "")]
pub trait InvalidNamespace {
    #[ink(message)]
    fn message(&self);
}

fn main() {}
