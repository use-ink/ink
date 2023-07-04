#[ink::event]
pub struct Event {
    #[cfg(feature = "std")]
    pub topic: [u8; 32],
}

fn main() {}