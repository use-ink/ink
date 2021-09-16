use ink_lang as ink;

#[ink::trait_definition]
pub trait OverlappingSelectors {
    #[ink(message, selector = 1)]
    fn selector1(&self);
    #[ink(message, selector = 1)]
    fn selector2(&self);
}

fn main() {}
