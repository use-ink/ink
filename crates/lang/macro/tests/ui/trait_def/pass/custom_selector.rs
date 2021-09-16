use ink_lang as ink;

#[ink::trait_definition]
pub trait CustomSelector {
    #[ink(message, selector = "0x00000001")]
    fn selector1(&self);
    #[ink(message, selector = "0x00000002")]
    fn selector1_mut(&mut self);

    #[ink(message, selector = 3)]
    fn selector2(&self);
    #[ink(message, selector = 4)]
    fn selector2_mut(&mut self);

    #[ink(message, selector = 0x0000_0005)]
    fn selector3(&self);
    #[ink(message, selector = 0x0000_0006)]
    fn selector3_mut(&mut self);
}

fn main() {}
