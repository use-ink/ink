use ink_lang as ink;

#[ink::trait_definition]
pub trait SimpleDefinition {
    #[ink(message)]
    fn simple(&self);
    #[ink(message)]
    fn simple_mut(&mut self);
}

fn main() {}
