#[ink::event]
#[ink(anonymous, anonymous)]
pub struct Event {
    #[ink(topic)]
    pub topic: [u8; 32],
}

fn main() {}