// `sol_name` is only supported in Solidity ABI compatibility mode.
#[ink::event(sol_name = "MyEvent")]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

fn main() {}
