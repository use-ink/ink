//! ZkSnarks Supports
//!
//! Provides type definitions and traits for the built-in cryptographic zksnarks

mod private {
    /// Seals the implementation of `CryptoHash` and `HashOutput`.
    pub trait Sealed {}
}

/// Default supports any types
pub trait Default {
    /// Default value
    fn default() -> Self;
}

impl Default for [u8; 64] {
    fn default() -> Self {
        [0; 64]
    }
}

/// The output type of a built-in cryptographic hash function.
pub trait CurvePointOutput: private::Sealed {
    /// The output type of the crypto hash.
    ///
    /// This should be a byte array with some constant size such as `[u8; 64]`.
    type Type: Default;
}

/// Types that are usable as built-in cryptographic hashes.
pub trait CurvePoint: CurvePointOutput + private::Sealed {
    /// Hashes the given raw byte input and copies the result into `output`.
    fn inflect(input: &[u8], output: &mut <Self as CurvePointOutput>::Type);
}

/// The AltBn128Add with 512-bit output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AltBn128Add {}

/// The AltBn128Mul with 512-bit output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AltBn128Mul {}

// /// The AltBn128Pairing with 512-bit output.
// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// pub enum AltBn128Pairing {}

impl private::Sealed for AltBn128Add {}
impl private::Sealed for AltBn128Mul {}
// impl private::Sealed for AltBn128Pairing {}

impl CurvePointOutput for AltBn128Add {
    type Type = [u8; 64];
}

impl CurvePointOutput for AltBn128Mul {
    type Type = [u8; 64];
}
