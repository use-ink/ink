use ink_lang as ink;

#[ink::trait_definition]
pub trait ManyOutputs {
    #[ink(message)]
    fn output0(&self);
    #[ink(message)]
    fn output0_mut(&mut self);

    #[ink(message)]
    fn output1(&self) -> i8;
    #[ink(message)]
    fn output1_mut(&mut self) -> i8;

    #[ink(message)]
    fn output2(&self) -> (i8, i16);
    #[ink(message)]
    fn output2_mut(&mut self) -> (i8, i16);

    #[ink(message)]
    fn output3(&self) -> (i8, i16, i32);
    #[ink(message)]
    fn output3_mut(&mut self) -> (i8, i16, i32);

    #[ink(message)]
    fn output4(&self) -> (i8, i16, i32, i64);
    #[ink(message)]
    fn output4_mut(&mut self) -> (i8, i16, i32, i64);
}

fn main() {}
