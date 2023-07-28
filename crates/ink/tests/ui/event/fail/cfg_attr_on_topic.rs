#[ink::event]
pub struct Event {
    #[cfg(feature = "std")]
    #[ink(topic)]
    pub topic: [u8; 32],
}

fn main() {}