#[ink::event(name = "MyEvent")]
#[ink(signature_topic = "1111111111111111111111111111111111111111111111111111111111111111")]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {
    // Custom signature topic (i.e `signature_topic = "..."`) takes precedence over `name` override
    const SIGNATURE_TOPIC: [u8; 32] = [0x11u8; 32];
    assert_eq!(<Event as ink::env::Event>::SIGNATURE_TOPIC, Some(SIGNATURE_TOPIC));
}
