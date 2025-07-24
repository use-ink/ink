#![allow(unexpected_cfgs)]

// Either no variants have explicitly specified discriminants,
// or all variants must have explicitly specified discriminants.

#[derive(ink::SolDecode, ink::SolEncode)]
enum SimpleEnum {
    One = 1,
    Two,
    Three,
}

fn main() {}
