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

use super::max;
use crate::traits::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_env::{
    AccountId,
    Hash,
};
use ink_prelude::{
    boxed::Box,
    string::String,
};
use ink_primitives::Key;

macro_rules! impl_layout_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl_always_packed_layout!($ty, deep: false);
            impl PackedLayout for $ty {
                #[inline(always)]
                fn pull_packed(&mut self, _at: &Key) {}
                #[inline(always)]
                fn push_packed(&self, _at: &Key) {}
                #[inline(always)]
                fn clear_packed(&self, _at: &Key) {}
            }
        )*
    };
}
#[rustfmt::skip]
impl_layout_for_primitive!(
    // We do not include `f32` and `f64` since Wasm contracts currently
    // do not support them since they are non deterministic. We might add them
    // to this list once we add deterministic support for those primitives.
    Key, Hash, AccountId, (),
    String,
    bool,
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
);

impl<T> SpreadLayout for Option<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = 1 + <T as SpreadLayout>::FOOTPRINT;
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

    fn push_spread(&self, ptr: &mut KeyPtr) {
        <u8 as SpreadLayout>::push_spread(&(self.is_some() as u8), ptr);
        if let Some(value) = self {
            <T as SpreadLayout>::push_spread(value, ptr);
        } else {
            ptr.advance_by(<T as SpreadLayout>::FOOTPRINT);
        }
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        // We do not really need the reference to 0 (zero)
        // in order to clean-up the `bool` value from the storage.
        // However the API is demanding a reference so we give it one.
        <u8 as SpreadLayout>::clear_spread(&0, ptr);
        if let Some(value) = self {
            <T as SpreadLayout>::clear_spread(value, ptr)
        } else {
            ptr.advance_by(<T as SpreadLayout>::FOOTPRINT);
        }
    }

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        match <u8 as SpreadLayout>::pull_spread(ptr) {
            0u8 => {
                ptr.advance_by(<T as SpreadLayout>::FOOTPRINT);
                None
            }
            1u8 => Some(<T as SpreadLayout>::pull_spread(ptr)),
            _ => unreachable!("invalid Option discriminant"),
        }
    }
}

impl<T> PackedLayout for Option<T>
where
    T: PackedLayout,
{
    #[inline]
    fn push_packed(&self, at: &Key) {
        if let Some(value) = self {
            <T as PackedLayout>::push_packed(value, at)
        }
    }

    #[inline]
    fn clear_packed(&self, at: &Key) {
        if let Some(value) = self {
            <T as PackedLayout>::clear_packed(value, at)
        }
    }

    #[inline]
    fn pull_packed(&mut self, at: &Key) {
        if let Some(value) = self {
            <T as PackedLayout>::pull_packed(value, at)
        }
    }
}

impl<T, E> SpreadLayout for Result<T, E>
where
    T: SpreadLayout,
    E: SpreadLayout,
{
    const FOOTPRINT: u64 = 1 + max(
        <T as SpreadLayout>::FOOTPRINT,
        <E as SpreadLayout>::FOOTPRINT,
    );
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
        || <E as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        match <u8 as SpreadLayout>::pull_spread(ptr) {
            0 => Ok(<T as SpreadLayout>::pull_spread(ptr)),
            1 => Err(<E as SpreadLayout>::pull_spread(ptr)),
            _ => unreachable!("invalid Result discriminant"),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        match self {
            Ok(value) => {
                <u8 as SpreadLayout>::push_spread(&0, ptr);
                <T as SpreadLayout>::push_spread(value, ptr);
            }
            Err(error) => {
                <u8 as SpreadLayout>::push_spread(&1, ptr);
                <E as SpreadLayout>::push_spread(error, ptr);
            }
        }
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        // Clear the discriminant, same for all variants.
        <u8 as SpreadLayout>::clear_spread(&0, ptr);
        match self {
            Ok(value) => {
                <T as SpreadLayout>::clear_spread(value, ptr);
            }
            Err(error) => {
                <E as SpreadLayout>::clear_spread(error, ptr);
            }
        }
    }
}

impl<T, E> PackedLayout for Result<T, E>
where
    T: PackedLayout,
    E: PackedLayout,
{
    #[inline]
    fn push_packed(&self, at: &Key) {
        match self {
            Ok(value) => <T as PackedLayout>::push_packed(value, at),
            Err(error) => <E as PackedLayout>::push_packed(error, at),
        }
    }

    #[inline]
    fn clear_packed(&self, at: &Key) {
        match self {
            Ok(value) => <T as PackedLayout>::clear_packed(value, at),
            Err(error) => <E as PackedLayout>::clear_packed(error, at),
        }
    }

    #[inline]
    fn pull_packed(&mut self, at: &Key) {
        match self {
            Ok(value) => <T as PackedLayout>::pull_packed(value, at),
            Err(error) => <E as PackedLayout>::pull_packed(error, at),
        }
    }
}

impl<T> SpreadLayout for Box<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = <T as SpreadLayout>::FOOTPRINT;
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Box::new(<T as SpreadLayout>::pull_spread(ptr))
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        <T as SpreadLayout>::push_spread(&*self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        <T as SpreadLayout>::clear_spread(&*self, ptr)
    }
}

impl<T> PackedLayout for Box<T>
where
    T: PackedLayout,
{
    #[inline]
    fn push_packed(&self, at: &Key) {
        <T as PackedLayout>::push_packed(&*self, at)
    }

    #[inline]
    fn clear_packed(&self, at: &Key) {
        <T as PackedLayout>::clear_packed(&*self, at)
    }

    #[inline]
    fn pull_packed(&mut self, at: &Key) {
        <T as PackedLayout>::pull_packed(&mut *self, at)
    }
}

#[cfg(test)]
mod tests {
    use crate::push_pull_works_for_primitive;
    use ink_env::AccountId;
    use ink_primitives::Key;

    push_pull_works_for_primitive!(bool, [false, true]);
    push_pull_works_for_primitive!(
        String,
        [Default::default(), String::from("Hello, World!")]
    );
    push_pull_works_for_primitive!(
        Key,
        [
            Key::from([0x00; 32]),
            Key::from([0x42; 32]),
            Key::from([0xFF; 32])
        ]
    );
    push_pull_works_for_primitive!(
        AccountId,
        [
            AccountId::from([0x00; 32]),
            AccountId::from([0x42; 32]),
            AccountId::from([0xFF; 32])
        ]
    );
    push_pull_works_for_primitive!(i8, [0, Default::default(), 1, i8::MIN, i8::MAX]);
    push_pull_works_for_primitive!(i16, [0, Default::default(), 2, i16::MIN, i16::MAX]);
    push_pull_works_for_primitive!(i32, [0, Default::default(), 3, i32::MIN, i32::MAX]);
    push_pull_works_for_primitive!(i64, [0, Default::default(), 4, i64::MIN, i64::MAX]);
    push_pull_works_for_primitive!(
        i128,
        [0, Default::default(), 5, i128::MIN, i128::MAX]
    );
    push_pull_works_for_primitive!(u8, [0, Default::default(), 10, u8::MIN, u8::MAX]);
    push_pull_works_for_primitive!(u16, [0, Default::default(), 20, u16::MIN, u16::MAX]);
    push_pull_works_for_primitive!(u32, [0, Default::default(), 30, u32::MIN, u32::MAX]);
    push_pull_works_for_primitive!(u64, [0, Default::default(), 40, u64::MIN, u64::MAX]);
    push_pull_works_for_primitive!(
        u128,
        [0, Default::default(), 50, u128::MIN, u128::MAX]
    );

    type OptionU8 = Option<u8>;
    push_pull_works_for_primitive!(OptionU8, [Some(13u8), None]);

    type ResultU8 = Result<u8, bool>;
    push_pull_works_for_primitive!(ResultU8, [Ok(13u8), Err(false)]);

    type BoxU8 = Box<u8>;
    push_pull_works_for_primitive!(BoxU8, [Box::new(27u8)]);

    type BoxOptionU8 = Box<Option<u8>>;
    push_pull_works_for_primitive!(BoxOptionU8, [Box::new(Some(27)), Box::new(None)]);
}
