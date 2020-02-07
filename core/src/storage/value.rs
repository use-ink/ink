// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

use crate::storage::{
    self,
    alloc::{
        Allocate,
        AllocateUsing,
        Initialize,
    },
    cell::SyncCell,
    Flush,
};

// Missing traits:
//
// - DerefMut
// - IndexMut
// - Borrow

/// A value on the storage.
///
/// This is a generic wrapper around a value in the contract storage.
/// It tries to model the wrapped type as close as possible so that using
/// it feels like using the underlying wrapped type.
///
/// While optionally implementing several common core traits, such as `PartialEq`,
/// `Debug`, `Add`, `ShlAssign`, `Deref` etc. it does not implement `DerefMut` or
/// `IndexMut` for security reasons.
///
/// For assigning new values or mutating the value inside of it either use
/// [`set`](struct.Value.html#method.set) or
/// [`mutate_with`](struct.Value.html#method.mutate_with).
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Value<T> {
    /// The cell of the storage value.
    cell: SyncCell<T>,
}

#[cfg(feature = "ink-generate-abi")]
impl<T> HasLayout for Value<T>
where
    T: Metadata + 'static,
{
    fn layout(&self) -> StorageLayout {
        LayoutStruct::new(Self::meta_type(), vec![LayoutField::of("cell", &self.cell)])
            .into()
    }
}

impl<T> AllocateUsing for Value<T> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            cell: SyncCell::allocate_using(alloc),
        }
    }
}

impl<T> Initialize for Value<T>
where
    T: Encode,
{
    type Args = T;

    #[inline]
    fn initialize(&mut self, args: Self::Args) {
        self.cell.set(args);
    }
}

impl<T> Value<T>
where
    T: scale::Codec + Default,
{
    /// Creates a new storage value initialized as its default value.
    ///
    /// # Safety
    ///
    /// The is unsafe because it does not check if the associated storage
    /// does not alias with storage allocated by other storage allocators.
    pub unsafe fn default_using<A>(alloc: &mut A) -> Self
    where
        A: storage::Allocator,
    {
        Self::allocate_using(alloc).initialize_into(Default::default())
    }
}

impl<T> Value<T>
where
    T: scale::Codec,
{
    /// Returns an immutable reference to the wrapped value.
    pub fn get(&self) -> &T {
        self.cell.get().unwrap()
    }

    /// Returns a mutable reference to the wrapped value.
    pub fn get_mut(&mut self) -> &mut T {
        self.cell.get_mut().unwrap()
    }

    /// Sets the wrapped value to the given value.
    pub fn set(&mut self, val: T) {
        self.cell.set(val);
    }
}

impl<T> Value<T>
where
    T: scale::Codec,
{
    /// Mutates the wrapped value inplace by the given closure.
    ///
    /// Returns a reference to the resulting value.
    pub fn mutate_with<F>(&mut self, f: F) -> &T
    where
        F: FnOnce(&mut T),
    {
        self.cell.mutate_with(f).unwrap()
    }
}

impl<T, R> AsRef<R> for Value<T>
where
    T: AsRef<R> + scale::Codec,
{
    fn as_ref(&self) -> &R {
        self.get().as_ref()
    }
}

impl<T> core::ops::Deref for Value<T>
where
    T: scale::Codec,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for Value<T>
where
    T: scale::Codec,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> Flush for Value<T>
where
    T: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.cell.flush()
    }
}

impl<T> Drop for Value<T> {
    #[inline]
    fn drop(&mut self) {
        self.cell.clear()
    }
}

macro_rules! impl_ops_for_value {
	(
		$trait_name:ident, $fn_name:ident,
		$trait_name_assign:ident, $fn_name_assign:ident;
		$tok:tt, $tok_eq:tt
	) => {
		impl<T> core::ops::$trait_name<T> for &Value<T>
		where
			T: core::ops::$trait_name<T> + Copy + scale::Codec,
		{
			type Output = <T as core::ops::$trait_name>::Output;

			fn $fn_name(self, rhs: T) -> Self::Output {
				*self.get() $tok rhs
			}
		}

		impl<T> core::ops::$trait_name for &Value<T>
		where
			T: core::ops::$trait_name<T> + Copy + scale::Codec,
		{
			type Output = <T as core::ops::$trait_name>::Output;

			fn $fn_name(self, rhs: Self) -> Self::Output {
				(*self.get()) $tok (*rhs.get())
			}
		}

		impl<T> core::ops::$trait_name_assign<T> for Value<T>
		where
			T: core::ops::$trait_name_assign<T> + scale::Codec,
		{
			fn $fn_name_assign(&mut self, rhs: T) {
				self.mutate_with(|val| (*val) $tok_eq rhs);
			}
		}

		impl<T> core::ops::$trait_name_assign<&Self> for Value<T>
		where
			T: core::ops::$trait_name_assign<T> + Copy + scale::Codec,
		{
			fn $fn_name_assign(&mut self, rhs: &Value<T>) {
				self.mutate_with(|val| (*val) $tok_eq *rhs.get());
			}
		}
	};
}

impl_ops_for_value!(Add, add, AddAssign, add_assign; +, +=);
impl_ops_for_value!(Sub, sub, SubAssign, sub_assign; -, -=);
impl_ops_for_value!(Mul, mul, MulAssign, mul_assign; *, *=);
impl_ops_for_value!(Div, div, DivAssign, div_assign; /, /=);
impl_ops_for_value!(Rem, rem, RemAssign, rem_assign; %, %=);

impl_ops_for_value!(BitAnd, bitand, BitAndAssign, bitand_assign; &, &=);
impl_ops_for_value!(BitOr, bitor, BitOrAssign, bitor_assign; |, |=);
impl_ops_for_value!(BitXor, bitxor, BitXorAssign, bitxor_assign; ^, ^=);

impl<T> core::ops::Neg for &Value<T>
where
    T: core::ops::Neg + Copy + scale::Codec,
{
    type Output = <T as core::ops::Neg>::Output;

    fn neg(self) -> Self::Output {
        -(*self.get())
    }
}

impl<T> core::ops::Not for &Value<T>
where
    T: core::ops::Not + Copy + scale::Codec,
{
    type Output = <T as core::ops::Not>::Output;

    fn not(self) -> Self::Output {
        !(*self.get())
    }
}

macro_rules! impl_shift_for_value {
	(
		$trait_name:ident, $fn_name:ident, $tok:tt;
		$trait_name_assign:ident, $fn_name_assign:ident, $tok_eq:tt
	) => {
		impl<T, R> core::ops::$trait_name<R> for &Value<T>
		where
			T: core::ops::$trait_name<R> + Copy + scale::Codec,
		{
			type Output = <T as core::ops::$trait_name<R>>::Output;

			fn $fn_name(self, rhs: R) -> Self::Output {
				(*self.get()) $tok rhs
			}
		}

		impl<T, R> core::ops::$trait_name_assign<R> for Value<T>
		where
			T: core::ops::$trait_name_assign<R> + Copy + scale::Codec,
		{
			fn $fn_name_assign(&mut self, rhs: R) {
				self.mutate_with(|value| (*value) $tok_eq rhs);
			}
		}
	};
}

impl_shift_for_value!(Shl, shl, <<; ShlAssign, shl_assign, <<=);
impl_shift_for_value!(Shr, shr, >>; ShrAssign, shr_assign, >>=);

impl<T, I> core::ops::Index<I> for Value<T>
where
    T: core::ops::Index<I> + scale::Codec,
{
    type Output = <T as core::ops::Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.get()[index]
    }
}

impl<T> PartialEq<T> for Value<T>
where
    T: PartialEq + scale::Codec,
{
    fn eq(&self, rhs: &T) -> bool {
        self.get().eq(rhs)
    }
}

impl<T> PartialEq for Value<T>
where
    T: PartialEq + scale::Codec,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.get().eq(rhs.get())
    }
}

impl<T> Eq for Value<T> where T: Eq + scale::Codec {}

use core::cmp::Ordering;

impl<T> PartialOrd<T> for Value<T>
where
    T: PartialOrd + scale::Codec,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl<T> PartialOrd<Value<T>> for Value<T>
where
    T: PartialOrd + scale::Codec,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<T> Ord for Value<T>
where
    T: Ord + scale::Codec,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(other.get())
    }
}

impl<T> core::hash::Hash for Value<T>
where
    T: core::hash::Hash + scale::Codec,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl<T> core::fmt::Display for Value<T>
where
    T: core::fmt::Display + scale::Codec,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.get().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::alloc::{
        AllocateUsing,
        BumpAlloc,
        Initialize,
    };
    use ink_primitives::Key;

    macro_rules! test_ops_impl {
        ( $test_name:ident, $tok:tt; $test_name_assign:ident, $tok_eq:tt) => {
            #[test]
            fn $test_name() {
                let (val1, val2, val3) = unsafe {
                    let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
                    let mut val1: Value<i32> = Value::allocate_using(&mut alloc);
                    let mut val2: Value<i32> = Value::allocate_using(&mut alloc);
                    let mut val3: Value<i32> = Value::allocate_using(&mut alloc);
                    val1.initialize(42);
                    val2.initialize(5);
                    val3.initialize(&val1 $tok &val2);
                    (val1, val2, val3)
                };
                // Check init values
                assert_eq!(val1.get(), &42);
                assert_eq!(val2.get(), &5);
                assert_eq!(val3.get(), &(42 $tok 5));
                // Operations with primitives
                assert_eq!(&val1 $tok 5, 42 $tok 5);
                // Operations with `Value<T>`
                assert_eq!(&val1 $tok &val2, 42 $tok 5);
            }

            #[test]
            fn $test_name_assign() {
                let (mut val1, mut copy, val2, val3) = unsafe {
                    let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
                    let mut val1: Value<i32> = Value::allocate_using(&mut alloc);
                    let mut copy: Value<i32> = Value::allocate_using(&mut alloc);
                    let mut val2: Value<i32> = Value::allocate_using(&mut alloc);
                    let mut val3: Value<i32> = Value::allocate_using(&mut alloc);
                    val1.initialize(42);
                    copy.initialize(42);
                    val2.initialize(13);
                    val3.initialize(42 $tok 13);
                    (val1, copy, val2, val3)
                };
                // Check init values
                assert_eq!(val1.get(), &42);
                assert_eq!(val2.get(), &13);
                assert_eq!(val3.get(), &(42 $tok 13));
                // Operation with primitives
                {
                    val1 $tok_eq 13;
                    assert_eq!(val1.get(), &(42 $tok 13));
                    assert_eq!(val1, val3);
                }
                // Operation between `Value<T>`
                {
                    copy $tok_eq &val2;
                    assert_eq!(copy.get(), &(42 $tok 13));
                    assert_eq!(copy, val3);
                }
            }
        };
    }

    test_ops_impl!(test_add   , +; test_add_assign   , +=);
    test_ops_impl!(test_sub   , -; test_sub_assign   , -=);
    test_ops_impl!(test_mul   , *; test_mul_assign   , *=);
    test_ops_impl!(test_div   , /; test_div_assign   , /=);
    test_ops_impl!(test_rem   , %; test_rem_assign   , %=);
    test_ops_impl!(test_bitand, &; test_bitand_assign, &=);
    test_ops_impl!(test_bitor , |; test_bitor_assign , |=);
    test_ops_impl!(test_bitxor, ^; test_bitxor_assign, ^=);

    macro_rules! test_unary_ops_impl {
        ( $test_name:ident, $trait_name:ident, $fn_name:ident, $tok:tt ) => {
            #[test]
            fn $test_name() {
                let (val1, val2) = unsafe {
                    let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
                    let mut val1: Value<i32> = Value::allocate_using(&mut alloc);
                    let mut val2: Value<i32> = Value::allocate_using(&mut alloc);
                    val1.initialize(42);
                    val2.initialize($tok 42);
                    (val1, val2)
                };
                // Check init values
                assert_eq!(val1.get(), &42);
                assert_eq!(val2.get(), &($tok 42));
                // Simple test
                assert_eq!($tok &val1, $tok 42);
                use core::ops::$trait_name;
                assert_eq!(val1.$fn_name(), $tok 42);
            }
        };
    }

    test_unary_ops_impl!(test_neg, Neg, neg, -);
    test_unary_ops_impl!(test_not, Not, not, !);

    #[test]
    fn test_shift() {
        let (mut value, result) = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            let mut value: Value<i32> = Value::allocate_using(&mut alloc);
            let mut result: Value<i32> = Value::allocate_using(&mut alloc);
            value.initialize(10);
            result.initialize(10 << 5);
            (value, result)
        };
        // Check init values
        assert_eq!(value.get(), &10);
        assert_eq!(result.get(), &(10 << 5));
        // Simple tests
        assert_eq!(&value << 5, 10 << 5);
        // Assign test
        value <<= 5;
        assert_eq!(&value, &result);
    }

    #[test]
    fn test_eq_ord() {
        let (val1, val2, val3) = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            let mut val1: Value<i32> = Value::allocate_using(&mut alloc);
            let mut val2: Value<i32> = Value::allocate_using(&mut alloc);
            let mut val3: Value<i32> = Value::allocate_using(&mut alloc);
            val1.initialize(42);
            val2.initialize(42);
            val3.initialize(1337);
            (val1, val2, val3)
        };
        // Eq & Ne
        assert!(val1 == val2);
        assert!(val2 != val3);
        // Less-Than
        assert!(!(val1 < val2));
        assert!(val2 < val3);
        assert!(val1 < val3);
        // Less-Than or Eq
        assert!(val1 <= val2);
        assert!(val2 <= val3);
        assert!(val1 <= val3);
    }

    #[test]
    fn test_index() {
        let val1 = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            let mut val1: Value<Vec<i32>> = Value::allocate_using(&mut alloc);
            val1.initialize(vec![2, 3, 5, 7, 11, 13]);
            val1
        };
        assert_eq!(val1[0], 2);
        assert_eq!(val1[1], 3);
        assert_eq!(val1[2], 5);
        assert_eq!(val1[3], 7);
        assert_eq!(val1[4], 11);
        assert_eq!(val1[5], 13);
    }
}
