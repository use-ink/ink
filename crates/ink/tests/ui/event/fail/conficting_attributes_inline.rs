#[ink::event]
#[ink(anonymous, signature_topic = "1111111111111111111111111111111111111111111111111111111111111111")]
pub struct Event {
    pub topic: [u8; 32],
}

fn main() {}
