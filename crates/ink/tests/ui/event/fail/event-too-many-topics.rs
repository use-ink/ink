#[ink::event]
pub struct Event {
    #[ink(topic)]
    pub topic_1: [u8; 32],
    #[ink(topic)]
    pub topic_2: [u8; 32],
    #[ink(topic)]
    pub topic_3: [u8; 32],
    #[ink(topic)]
    pub topic_4: [u8; 32],
    pub field_1: u32,
}

fn main() {}
