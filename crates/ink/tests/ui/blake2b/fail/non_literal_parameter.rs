const INPUT: &str = "test";
const _: [u8; 32] = ink::blake2x256!(INPUT);

fn main() {}
