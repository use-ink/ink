use ink_lang as ink;

#[ink::trait_definition]
pub trait InvalidSelector {
    #[ink(message, selector = 0xC0DECAFE)]
    fn invalid_selector(&self);
}

fn main() {}
