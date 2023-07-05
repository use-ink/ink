#[ink::event]
pub struct Event {
    #[cfg(feature = "std")]
    pub a: [u8; 32],
}

fn main() {}