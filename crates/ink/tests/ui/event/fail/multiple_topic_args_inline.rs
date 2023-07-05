#[ink::event]
pub struct Event {
    #[ink(topic, topic)]
    pub topic: [u8; 32],
}

fn main() {}