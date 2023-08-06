#[ink::event]
pub enum Event {
    Variant1 {
        field_1: i8,
        #[ink(topic)]
        field_2: i16,
    }
}

fn main() {}