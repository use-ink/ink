#![allow(unexpected_cfgs)]

// Enums are encoded as `u8` (i.e. `uint8` in Solidity ABI encoding),
// so the maximum allowed discriminant values is `u8::MAX` (i.e. `255`).

#[derive(ink::SolDecode, ink::SolEncode)]
enum SimpleEnum {
    Small = 1,
    Medium = 50,
    Large = 100,
    TooLarge = 256,
}

fn main() {}
