use ink_lang as ink;

#[ink::trait_definition]
pub trait CustomSelector {
    #[ink(message, selector = 1)]
    fn selector1(&self);
    #[ink(message, selector = 2)]
    fn selector1_mut(&mut self);

    #[ink(message, selector = 0x0000_0003)]
    fn selector2(&self);
    #[ink(message, selector = 0x0000_0004)]
    fn selector2_mut(&mut self);
}

fn main() {}
