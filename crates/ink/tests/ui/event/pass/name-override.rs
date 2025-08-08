#[ink::event(name = "MyEvent")]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {
    // Event signature topic computation uses the `name` override.
    const SIGNATURE_TOPIC: [u8; 32] = ::ink::blake2x256!("MyEvent([u8;32],u32)");
    assert_eq!(<Event as ink::env::Event>::SIGNATURE_TOPIC, Some(SIGNATURE_TOPIC));

    // Ensures `name` override is used in ink! metadata.
    let event_specs = ink::collect_events();
    assert_eq!(event_specs.len(), 1);
    assert_eq!(*event_specs[0].label(), "MyEvent");
}
