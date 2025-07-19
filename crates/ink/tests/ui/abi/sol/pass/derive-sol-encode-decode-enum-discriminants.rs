#![allow(unexpected_cfgs)]

use ink::{SolEncode, SolDecode};

#[derive(Debug, PartialEq, Eq, ink::SolDecode, ink::SolEncode)]
enum SimpleEnum {
    Two = 2,
    Three = 3,
    One = 1,
}

fn main() {
    // `SimpleEnum::One` is encoded as `1`
    let value = SimpleEnum::One;
    let encoded = SolEncode::encode(&1u8);
    assert_eq!(SolEncode::encode(&value), encoded);
    let decoded: SimpleEnum = SolDecode::decode(&encoded).unwrap();
    assert_eq!(value, decoded);

    // `SimpleEnum::Two` is encoded as `2`
    let value = SimpleEnum::Two;
    let encoded = SolEncode::encode(&2u8);
    assert_eq!(SolEncode::encode(&value), encoded);
    let decoded: SimpleEnum = SolDecode::decode(&encoded).unwrap();
    assert_eq!(value, decoded);

    // `SimpleEnum::Three` is encoded as `3`
    let value = SimpleEnum::Three;
    let encoded = SolEncode::encode(&3u8);
    assert_eq!(SolEncode::encode(&value), encoded);
    let decoded: SimpleEnum = SolDecode::decode(&encoded).unwrap();
    assert_eq!(value, decoded);
}
