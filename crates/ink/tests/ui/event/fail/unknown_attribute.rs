#[ink::event]
#[ink(anonymous)]
#[ink(my_arg)]
pub struct Event {
    pub topic: [u8; 32],
}

fn main() {}
