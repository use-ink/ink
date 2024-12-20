#[ink::event]
#[ink(signature_topic = "1111111111111111111111111111111111111111111111111111111111111111")]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {}
