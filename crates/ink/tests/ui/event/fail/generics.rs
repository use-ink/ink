#[derive(ink::Event)]
pub struct Event<T> {
    #[ink(topic)]
    pub topic: T,
}

fn main() {}