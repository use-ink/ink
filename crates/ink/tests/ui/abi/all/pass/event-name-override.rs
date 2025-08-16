#[ink::event(name = "MyEvent")]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {
    // ink! event signature topic computation uses the `name` override.
    const SIGNATURE_TOPIC_INK: [u8; 32] = ::ink::blake2x256!("MyEvent([u8;32],u32)");
    assert_eq!(<Event as ink::env::Event<ink::abi::Ink>>::SIGNATURE_TOPIC, Some(SIGNATURE_TOPIC_INK));

    // Solidity event signature topic computation uses the `name` override.
    const SIGNATURE_TOPIC_SOL: [u8; 32] = ::ink::keccak_256!("MyEvent(uint8[32],uint32)");
    assert_eq!(<Event as ink::env::Event<ink::abi::Sol>>::SIGNATURE_TOPIC, Some(SIGNATURE_TOPIC_SOL));

    // Ensures `name` override is used in ink! metadata.
    let event_specs = ink::collect_events();
    assert_eq!(event_specs.len(), 1);
    assert_eq!(*event_specs[0].label(), "MyEvent");

    // Ensures `name` override is used in Solidity metadata.
    let event_specs = ink::collect_events_sol();
    assert_eq!(event_specs.len(), 1);
    assert_eq!(event_specs[0].name, "MyEvent");
}
