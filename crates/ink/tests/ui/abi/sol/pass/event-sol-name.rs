#[ink::event(sol_name = "MyEvent")]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {
    // Ensures `sol_name` is used in Solidity metadata.
    let event_specs = ink::collect_events_sol();
    assert_eq!(event_specs.len(), 1);
    assert_eq!(event_specs[0].name, "MyEvent");
}
