#![allow(unexpected_cfgs)]

use ink::{SolEncode, SolDecode};

#[derive(Debug, PartialEq, Eq, ink::SolDecode, ink::SolEncode)]
enum SimpleEnum {
    First,
    Second,
    Third,
}

fn main() {
    // `SimpleEnum::First` is encoded as `0`
    let value = SimpleEnum::First;
    let encoded = SolEncode::encode(&0u8);
    assert_eq!(SolEncode::encode(&value), encoded);
    let decoded: SimpleEnum = SolDecode::decode(&encoded).unwrap();
    assert_eq!(value, decoded);

    // `SimpleEnum::Second` is encoded as `1`
    let value = SimpleEnum::Second;
    let encoded = SolEncode::encode(&1u8);
    assert_eq!(SolEncode::encode(&value), encoded);
    let decoded: SimpleEnum = SolDecode::decode(&encoded).unwrap();
    assert_eq!(value, decoded);

    // `SimpleEnum::Third` is encoded as `2`
    let value = SimpleEnum::Third;
    let encoded = SolEncode::encode(&2u8);
    assert_eq!(SolEncode::encode(&value), encoded);
    let decoded: SimpleEnum = SolDecode::decode(&encoded).unwrap();
    assert_eq!(value, decoded);
}
