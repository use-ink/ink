use ink_lang as ink;

const INPUT: &str = "test";
const _: [u8; 32] = ink::selector_id!(INPUT);

fn main() {}
