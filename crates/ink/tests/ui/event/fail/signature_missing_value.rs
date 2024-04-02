#[ink::event(signature_topic)]
pub struct Event {
    pub topic: [u8; 32],
}

fn main() {}
