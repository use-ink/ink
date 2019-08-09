// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::byte_utils;
use scale::{
    Decode,
    Encode,
};

/// Typeless generic key into contract storage.
///
/// Can be compared to a raw pointer featuring pointer arithmetic.
///
/// # Note
///
/// This is the most low-level method to access contract storage.
///
/// # Unsafe
///
/// - Does not restrict ownership.
/// - Can read and write to any storage location.
/// - Does not synchronize between main memory and contract storage.
/// - Violates Rust's mutability and immutability guarantees.
///
/// Prefer using types found in `collections` or `Synced` type.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Key(pub [u8; 32]);

impl core::fmt::Debug for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Key(")?;
        <Self as core::fmt::Display>::fmt(self, f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl core::fmt::Display for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x")?;
        if f.alternate() {
            let bytes = self.as_bytes();
            write!(
                f,
                "{:02X}{:02X}_{:02X}{:02X}_……_{:02X}{:02X}_{:02X}{:02X}",
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[28],
                bytes[29],
                bytes[30],
                bytes[31],
            )?;
        } else {
            for (n, byte) in self.as_bytes().iter().enumerate() {
                write!(f, "{:02X}", byte)?;
                if n % 4 == 0 && n != 32 {
                    write!(f, "_")?;
                }
            }
        }
        Ok(())
    }
}

impl Key {
    /// Returns the byte slice of this key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the mutable byte slice of this key.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl core::ops::Sub for Key {
    type Output = KeyDiff;

    fn sub(self, rhs: Self) -> KeyDiff {
        let mut lhs = self;
        let mut rhs = rhs;
        byte_utils::negate_bytes(rhs.as_bytes_mut());
        byte_utils::bytes_add_bytes(lhs.as_bytes_mut(), rhs.as_bytes());
        KeyDiff(lhs.0)
    }
}

/// The difference between two keys.
///
/// This is the result of substracting one key from another.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyDiff([u8; 32]);

macro_rules! impl_try_to_prim {
	(
		$( #[$attr:meta] )*
		$name:ident, $prim:ty, $conv:ident
	) => {
		impl KeyDiff {
			$( #[$attr] )*
			pub fn $name(&self) -> Option<$prim> {
				const KEY_BYTES: usize = 32;
				const PRIM_BYTES: usize = core::mem::size_of::<$prim>();
				if self.as_bytes().iter().take(KEY_BYTES - PRIM_BYTES).any(|&byte| byte != 0x0) {
					return None
				}
				let value = <$prim>::from_be_bytes(
					*byte_utils::$conv(&self.as_bytes()[(KEY_BYTES - PRIM_BYTES)..KEY_BYTES])
						.unwrap()
				);
				Some(value)
			}
		}
	};
}

impl_try_to_prim!(
    /// Tries to convert the key difference to a `u32` if possible.
    ///
    /// Returns `None` if the resulting value is out of bounds.
    try_to_u32,
    u32,
    slice4_as_array4
);
impl_try_to_prim!(
    /// Tries to convert the key difference to a `u64` if possible.
    ///
    /// Returns `None` if the resulting value is out of bounds.
    try_to_u64,
    u64,
    slice8_as_array8
);
impl_try_to_prim!(
    /// Tries to convert the key difference to a `u128` if possible.
    ///
    /// Returns `None` if the resulting value is out of bounds.
    try_to_u128,
    u128,
    slice16_as_array16
);

impl KeyDiff {
    /// Returns the byte slice of this key difference.
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

macro_rules! impl_add_sub_for_key {
    ( $prim:ty ) => {
        impl core::ops::Add<$prim> for Key {
            type Output = Self;

            fn add(self, rhs: $prim) -> Self::Output {
                let mut result = self;
                result += rhs;
                result
            }
        }

        impl core::ops::AddAssign<$prim> for Key {
            fn add_assign(&mut self, rhs: $prim) {
                byte_utils::bytes_add_bytes(self.as_bytes_mut(), &(rhs.to_be_bytes()));
            }
        }

        impl core::ops::Sub<$prim> for Key {
            type Output = Self;

            fn sub(self, rhs: $prim) -> Self::Output {
                let mut result = self;
                result -= rhs;
                result
            }
        }

        impl core::ops::SubAssign<$prim> for Key {
            fn sub_assign(&mut self, rhs: $prim) {
                byte_utils::bytes_sub_bytes(self.as_bytes_mut(), &rhs.to_be_bytes());
            }
        }
    };
}

impl_add_sub_for_key!(u32);
impl_add_sub_for_key!(u64);
impl_add_sub_for_key!(u128);

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;

    use crate::{
        env,
        test_utils::run_test,
    };

    #[test]
    fn store_load_clear() {
        run_test(|| {
            let key = Key([0x42; 32]);
            assert_eq!(unsafe { env::load(key) }, None);
            unsafe {
                env::store(key, &[0x5]);
            }
            assert_eq!(unsafe { env::load(key) }, Some(vec![0x5]));
            unsafe {
                env::clear(key);
            }
            assert_eq!(unsafe { env::load(key) }, None);
        })
    }

    #[test]
    fn key_add() {
        run_test(|| {
            let key00 = Key([0x0; 32]);
            let key05 = key00 + 5_u32; // -> 5
            let key10 = key00 + 10_u32; // -> 10         | same as key55
            let key55 = key05 + 5_u32; // -> 5 + 5 = 10 | same as key10
            unsafe {
                env::store(key55, &[42]);
            }
            assert_eq!(unsafe { env::load(key10) }, Some(vec![42]));
            unsafe {
                env::store(key10, &[13, 37]);
            }
            assert_eq!(unsafe { env::load(key55) }, Some(vec![13, 37]));
        })
    }

    #[test]
    fn key_add_sub() {
        run_test(|| {
            let key0a = Key([0x0; 32]);
            unsafe {
                env::store(key0a, &[0x01]);
            }
            let key1a = key0a + 1337_u32;
            unsafe {
                env::store(key1a, &[0x02]);
            }
            let key2a = key0a + 42_u32;
            unsafe {
                env::store(key2a, &[0x03]);
            }
            let key3a = key0a + 52_u32;
            let key2b = key3a - 10_u32;
            assert_eq!(unsafe { env::load(key2b) }, Some(vec![0x03]));
            let key1b = key2b - 42_u32;
            assert_eq!(unsafe { env::load(key1b) }, Some(vec![0x01]));
            let key0b = key1b + 2000_u32 - 663_u32;
            assert_eq!(unsafe { env::load(key0b) }, Some(vec![0x02]));
        })
    }

    #[test]
    fn key_sub() {
        run_test(|| {
            assert_eq!(Key([0x42; 32]) - 0_u32, Key([0x42; 32]));
            assert_eq!(Key([0x00; 32]) - 1_u32, Key([0xFF; 32]));
            assert_eq!(
                Key([0x01; 32]) - 1_u32,
                Key([
                    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00
                ])
            );
            {
                let key_u32_max_value = Key([
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
                ]);
                assert_eq!(key_u32_max_value - u32::max_value(), Key([0x00; 32]));
            }
            {
                let key_a = Key([
                    0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x10, 0x20, 0x30,
                    0x40, 0x50, 0x60, 0x70, 0x80, 0xA0, 0xB1, 0xC2, 0xD3, 0xE4, 0xF5,
                    0x06, 0x17, 0x00, 0x22, 0x44, 0x66, 0x88, 0xAA, 0xCC, 0xEE,
                ]);
                let b: u32 = 0xFA09_51C3;
                let expected_b = Key([
                    0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x10, 0x20, 0x30,
                    0x40, 0x50, 0x60, 0x70, 0x80, 0xA0, 0xB1, 0xC2, 0xD3, 0xE4, 0xF5,
                    0x06, 0x17, 0x00, 0x22, 0x44, 0x65, 0x8E, 0xA1, 0x7B, 0x2B,
                ]);
                assert_eq!(key_a - b, expected_b);
                let c: u64 = 0xFBDC_BEEF_9999_1234;
                let expected_c = Key([
                    0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x10, 0x20, 0x30,
                    0x40, 0x50, 0x60, 0x70, 0x80, 0xA0, 0xB1, 0xC2, 0xD3, 0xE4, 0xF5,
                    0x06, 0x16, 0x04, 0x45, 0x85, 0x76, 0xEF, 0x11, 0xBA, 0xBA,
                ]);
                assert_eq!(key_a - c, expected_c);
            }
        })
    }

    #[test]
    fn as_bytes() {
        run_test(|| {
            let mut key = Key([0x42; 32]);
            assert_eq!(key.as_bytes(), &[0x42; 32]);
            assert_eq!(key.as_bytes_mut(), &mut [0x42; 32]);
        })
    }

    #[test]
    fn key_diff() {
        run_test(|| {
            let key1 = Key([0x0; 32]);
            let key2 = key1 + 0x42_u32;
            let key3 = key1 + u32::max_value() + 1_u32;
            let key4 = key1 + u64::max_value();
            assert_eq!(
                key2 - key1,
                KeyDiff([
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42,
                ])
            );
            assert_eq!((key2 - key1).try_to_u32(), Some(0x42));
            assert_eq!((key2 - key1).try_to_u64(), Some(0x42));
            assert_eq!(
                key3 - key1,
                KeyDiff([
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
                ])
            );
            assert_eq!((key3 - key1).try_to_u32(), None);
            assert_eq!(
                (key3 - key1).try_to_u64(),
                Some(u32::max_value() as u64 + 1)
            );
            assert_eq!(
                key4 - key1,
                KeyDiff([
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ])
            );
            assert_eq!((key4 - key1).try_to_u32(), None);
            assert_eq!((key4 - key1).try_to_u64(), Some(u64::max_value()));
            assert_eq!(
                key4 - key3,
                KeyDiff([
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF,
                ])
            );
            assert_eq!((key4 - key1).try_to_u32(), None);
            assert_eq!(
                (key4 - key3).try_to_u64(),
                Some(u64::max_value() - (u32::max_value() as u64 + 1))
            );
        })
    }
}
