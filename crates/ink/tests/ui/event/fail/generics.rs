#[derive(ink::Event, scale::Encode)]
pub struct Event<T> {
    #[ink(topic)]
    pub topic: T,
}

fn main() {}