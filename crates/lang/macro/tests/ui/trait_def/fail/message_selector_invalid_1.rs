use ink_lang as ink;

#[ink::trait_definition]
pub trait InvalidSelector {
    #[ink(message, selector = true)]
    fn invalid_selector(&self);
}

fn main() {}
