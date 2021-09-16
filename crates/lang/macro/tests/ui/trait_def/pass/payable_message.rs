use ink_lang as ink;

#[ink::trait_definition]
pub trait PayableDefinition {
    #[ink(message, payable)]
    fn payable(&self);
    #[ink(message, payable)]
    fn payable_mut(&mut self);
}

fn main() {}
