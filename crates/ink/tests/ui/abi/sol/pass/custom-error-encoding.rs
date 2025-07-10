#![allow(unexpected_cfgs)]

use ink::sol::{SolErrorEncode, SolErrorDecode};

// Equivalent to a Solidity custom error with no params.
#[derive(Debug, PartialEq, Eq, ink::SolErrorDecode, ink::SolErrorEncode)]
pub struct UnitError;

// Equivalent to a Solidity custom error with params.
#[derive(Debug, PartialEq, Eq, ink::SolErrorDecode, ink::SolErrorEncode)]
struct ErrorWithParams(bool);

// Equivalent to multiple Solidity custom errors, one for each variant.
#[derive(Debug, PartialEq, Eq, ink::SolErrorDecode, ink::SolErrorEncode)]
pub enum MultipleErrors {
    UnitError,
    ErrorWithParams(bool)
}

fn main() {
    // `keccak256("UnitError()")` == `0xe930c64c`
    let error = UnitError;
    let encoded = vec![0xe9, 0x30, 0xc6, 0x4c];
    assert_eq!(SolErrorEncode::encode(&error), encoded);
    let decoded: UnitError = SolErrorDecode::decode(&encoded).unwrap();
    assert_eq!(error, decoded);

    // `keccak256("ErrorWithParams(bool)")` == `0xac3a6266`
    let error = ErrorWithParams(true);
    let mut encoded = vec![0xac, 0x3a, 0x62, 0x66];
    // SolEncode(true) i.e. `0x1` preceded by 31 `0x0`
    let mut encoded_params = [0x0; 32];
    encoded_params[31] = 0x1;
    encoded.extend(encoded_params);
    assert_eq!(SolErrorEncode::encode(&error), encoded);
    let decoded: ErrorWithParams = SolErrorDecode::decode(&encoded).unwrap();
    assert_eq!(error, decoded);

    // `MultipleErrors::UnitError` is encoded/decoded as `keccak256("UnitError()")` == `0xe930c64c`
    let error = MultipleErrors::UnitError;
    let encoded = vec![0xe9, 0x30, 0xc6, 0x4c];
    assert_eq!(SolErrorEncode::encode(&error), encoded);
    let decoded: MultipleErrors = SolErrorDecode::decode(&encoded).unwrap();
    assert_eq!(error, decoded);

    // `MultipleErrors::ErrorWithParams(bool)` is encoded/decoded as
    // `keccak256("ErrorWithParams(bool)")` == `0xac3a6266`
    let error = MultipleErrors::ErrorWithParams(true);
    let mut encoded = vec![0xac, 0x3a, 0x62, 0x66];
    // SolEncode(true) i.e. `0x1` preceded by 31 `0x0`
    let mut encoded_params = [0x0; 32];
    encoded_params[31] = 0x1;
    encoded.extend(encoded_params);
    assert_eq!(SolErrorEncode::encode(&error), encoded);
    let decoded: MultipleErrors = SolErrorDecode::decode(&encoded).unwrap();
    assert_eq!(error, decoded);
}
