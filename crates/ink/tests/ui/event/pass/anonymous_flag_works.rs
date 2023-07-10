#[ink::event(anonymous = true)]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {}