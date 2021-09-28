use ink_lang as ink;

#[ink::trait_definition]
pub trait InvalidPayable {
    #[ink(message, payable = true)]
    fn invalid_payable(&self);
}

fn main() {}
