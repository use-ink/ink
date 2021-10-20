use ink_lang as ink;

#[ink::trait_definition]
pub trait ManyInputs {
    #[ink(message)]
    fn input0(&self);
    #[ink(message)]
    fn input0_mut(&mut self);

    #[ink(message)]
    fn input1(&self, a: i8);
    #[ink(message)]
    fn input1_mut(&mut self, a: i8);

    #[ink(message)]
    fn input2(&self, a: i8, b: i16);
    #[ink(message)]
    fn input2_mut(&mut self, a: i8, b: i16);

    #[ink(message)]
    fn input3(&self, a: i8, b: i16, c: i32);
    #[ink(message)]
    fn input3_mut(&mut self, a: i8, b: i16, c: i32);

    #[ink(message)]
    fn input4(&self, a: i8, b: i16, c: i32, d: i64);
    #[ink(message)]
    fn input4_mut(&mut self, a: i8, b: i16, c: i32, d: i64);
}

fn main() {}
