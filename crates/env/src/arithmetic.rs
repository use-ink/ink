// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Primitive traits for runtime arithmetic, copied from substrate

use core::{
    convert::{
        TryFrom,
        TryInto,
    },
    ops::{
        Add,
        AddAssign,
        Div,
        DivAssign,
        Mul,
        MulAssign,
        Sub,
        SubAssign,
    },
};
use num_traits::{
    checked_pow,
    Bounded,
    CheckedMul,
    One,
    Unsigned,
    Zero,
};

/// Types that allow for simple arithmetic operations.
///
/// Subset of all trait bounds copied over from what Substrate defines
/// for its `BaseArithmetic` types. We can extend this in the future
/// if needed.
pub trait BaseArithmetic:
    Sized
    + From<u8>
    + Bounded
    + Ord
    + PartialOrd<Self>
    + Zero
    + One
    + Bounded
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + CheckedMul
    + Saturating
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + TryFrom<u128>
    + TryFrom<usize>
    + TryInto<u16>
    + TryInto<u32>
    + TryInto<u64>
    + TryInto<u128>
    + TryInto<usize>
// Further trait bounds from the original BaseArithmetic trait
// that we could use to extend ink!'s BaseArithmetic trait.
//
// UniqueSaturatedInto<u8> +
// UniqueSaturatedInto<u16> +
// UniqueSaturatedInto<u32> +
// UniqueSaturatedInto<u64> +
// UniqueSaturatedInto<u128> +
// UniqueSaturatedFrom<u64> +
// UniqueSaturatedFrom<u128> +
// Shl<u32, Output = Self> +
// Shr<u32, Output = Self> +
// CheckedAdd +
// CheckedSub +
// CheckedDiv +
// CheckedShl +
// CheckedShr +
// IntegerSquareRoot +
{
}

impl<T> BaseArithmetic for T where
    T: Sized
        + From<u8>
        + Bounded
        + Ord
        + PartialOrd<Self>
        + Zero
        + One
        + Add<Self, Output = Self>
        + AddAssign<Self>
        + Sub<Self, Output = Self>
        + SubAssign<Self>
        + Mul<Self, Output = Self>
        + MulAssign<Self>
        + Div<Self, Output = Self>
        + DivAssign<Self>
        + CheckedMul
        + Saturating
        + TryFrom<u16>
        + TryFrom<u32>
        + TryFrom<u64>
        + TryFrom<u128>
        + TryFrom<usize>
        + TryInto<u16>
        + TryInto<u32>
        + TryInto<u64>
        + TryInto<u128>
        + TryInto<usize>
{
}

/// A meta trait for arithmetic (copied from substrate).
///
/// Arithmetic types do all the usual stuff you'd expect numbers to do. They are guaranteed to
/// be able to represent at least `u32` values without loss, hence the trait implies `From<u32>`
/// and smaller integers. All other conversions are fallible.
pub trait AtLeast32Bit: BaseArithmetic + From<u16> + From<u32> {}

impl<T> AtLeast32Bit for T where T: BaseArithmetic + From<u16> + From<u32> {}

/// A meta trait for arithmetic.  Same as [`AtLeast32Bit `], but also bounded to be unsigned.
pub trait AtLeast32BitUnsigned: AtLeast32Bit + Unsigned {}

impl<T> AtLeast32BitUnsigned for T where T: AtLeast32Bit + Unsigned {}

/// Saturating arithmetic operations, returning maximum or minimum values instead of overflowing.
pub trait Saturating {
    /// Saturating addition. Compute `self + rhs`, saturating at the numeric bounds instead of
    /// overflowing.
    fn saturating_add(self, rhs: Self) -> Self;

    /// Saturating subtraction. Compute `self - rhs`, saturating at the numeric bounds instead of
    /// overflowing.
    fn saturating_sub(self, rhs: Self) -> Self;

    /// Saturating multiply. Compute `self * rhs`, saturating at the numeric bounds instead of
    /// overflowing.
    fn saturating_mul(self, rhs: Self) -> Self;

    /// Saturating exponentiation. Compute `self.pow(exp)`, saturating at the numeric bounds
    /// instead of overflowing.
    fn saturating_pow(self, exp: usize) -> Self;
}

impl<T> Saturating for T
where
    T: Clone + Zero + One + PartialOrd + CheckedMul + Bounded + num_traits::Saturating,
{
    fn saturating_add(self, o: Self) -> Self {
        <Self as num_traits::Saturating>::saturating_add(self, o)
    }

    fn saturating_sub(self, o: Self) -> Self {
        <Self as num_traits::Saturating>::saturating_sub(self, o)
    }

    fn saturating_mul(self, o: Self) -> Self {
        self.checked_mul(&o).unwrap_or_else(|| {
            if (self < T::zero()) != (o < T::zero()) {
                Bounded::min_value()
            } else {
                Bounded::max_value()
            }
        })
    }

    fn saturating_pow(self, exp: usize) -> Self {
        let neg = self < T::zero() && exp % 2 != 0;
        checked_pow(self, exp).unwrap_or_else(|| {
            if neg {
                Bounded::min_value()
            } else {
                Bounded::max_value()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Saturating;

    #[test]
    fn saturating_add() {
        assert_eq!(
            u64::max_value(),
            Saturating::saturating_add(u64::max_value(), 1)
        )
    }

    #[test]
    fn saturatiung_sub() {
        assert_eq!(
            u64::min_value(),
            Saturating::saturating_sub(u64::min_value(), 1)
        )
    }

    #[test]
    fn saturating_mul() {
        assert_eq!(
            u64::max_value(),
            Saturating::saturating_mul(u64::max_value(), 2)
        );
        assert_eq!(
            i64::max_value(),
            Saturating::saturating_mul(i64::max_value(), 2)
        );
        assert_eq!(
            i64::min_value(),
            Saturating::saturating_mul(i64::min_value(), 2)
        );
        assert_eq!(
            i64::min_value(),
            Saturating::saturating_mul(2, i64::min_value())
        );
    }

    #[test]
    fn saturating_pow() {
        assert_eq!(
            u64::max_value(),
            Saturating::saturating_pow(u64::max_value(), 2)
        );
        assert_eq!(
            i64::max_value(),
            Saturating::saturating_pow(i64::min_value(), 2)
        );
        assert_eq!(
            i64::min_value(),
            Saturating::saturating_pow(i64::min_value(), 3)
        );
    }
}
