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

use cfg_if::cfg_if;
use core::{
    fmt::{
        self,
        Debug,
        Display,
        Formatter,
    },
    ops::AddAssign,
};
#[cfg(feature = "std")]
use scale_info::{
    build::Fields,
    Path,
    Type,
    TypeInfo,
};

/// A key into the smart contract storage.
///
/// # Note
///
/// - The storage of an ink! smart contract can be viewed as a key-value store.
/// - In order to manipulate its storage an ink! smart contract is required
///   to indicate the respective cells using this primitive type.
/// - The `Key` type can be compared to a raw pointer and also allows operations
///   similar to pointer arithmetic.
/// - Users usually should not have to deal with this low-level primitive themselves
///   and instead use the more high-level primitives provided by the `ink_storage`
///   crate.
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key([u8; 32]);

impl From<[u8; 32]> for Key {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8; 32]> for Key {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsMut<[u8; 32]> for Key {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl Key {
    fn write_bytes(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "0x")?;
        let bytes = self.as_ref();
        let len_bytes = bytes.len();
        let len_chunk = 4;
        let len_chunks = len_bytes / len_chunk;
        for i in 0..len_chunks {
            let offset = i * len_chunk;
            write!(
                f,
                "_{:02X}{:02X}{:02X}{:02X}",
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3]
            )?;
        }
        Ok(())
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Key(")?;
        self.write_bytes(f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.write_bytes(f)
    }
}

impl Key {
    /// Reinterprets the underlying bytes of the key as `&[u64; 4]`.
    ///
    /// # Safety
    ///
    /// This is only safe to do on little-endian systems therefore
    /// this function is only enabled on these platforms.
    #[cfg(target_endian = "little")]
    fn reinterpret_as_u64x4(&self) -> &[u64; 4] {
        // SAFETY: Conversion is only safe on little endian architectures.
        unsafe { &*(&self.0 as *const [u8; 32] as *const [u64; 4]) }
    }

    /// Reinterprets the underlying bytes of the key as `&mut [u64; 4]`.
    ///
    /// # Safety
    ///
    /// This is only safe to do on little-endian systems therefore
    /// this function is only enabled on these platforms.
    #[cfg(target_endian = "little")]
    fn reinterpret_as_u64x4_mut(&mut self) -> &mut [u64; 4] {
        // SAFETY: Conversion is only safe on little endian architectures.
        unsafe { &mut *(&mut self.0 as *mut [u8; 32] as *mut [u64; 4]) }
    }
}

impl scale::Encode for Key {
    #[inline(always)]
    fn size_hint(&self) -> usize {
        32
    }

    #[inline]
    fn encode_to<O>(&self, output: &mut O)
    where
        O: scale::Output + ?Sized,
    {
        output.write(self.as_ref());
    }

    #[inline]
    fn using_encoded<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(self.as_ref())
    }

    #[inline(always)]
    fn encoded_size(&self) -> usize {
        self.size_hint()
    }
}

impl scale::EncodeLike<[u8; 32]> for Key {}

impl scale::Decode for Key {
    #[inline]
    fn decode<I>(input: &mut I) -> Result<Self, scale::Error>
    where
        I: scale::Input,
    {
        let bytes = <[u8; 32] as scale::Decode>::decode(input)?;
        Ok(Self::from(bytes))
    }

    #[inline(always)]
    fn encoded_fixed_size() -> Option<usize> {
        Some(32)
    }
}

#[cfg(feature = "std")]
impl TypeInfo for Key {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Key", "ink_primitives"))
            .composite(
                Fields::unnamed().field(|f| f.ty::<[u8; 32]>().type_name("[u8; 32]")),
            )
    }
}

impl Key {
    /// Adds the `u64` value to the `Key`.
    ///
    /// # Note
    ///
    /// This implementation is heavily optimized for little-endian Wasm platforms.
    ///
    /// # Developer Note
    ///
    /// Since we are operating on little-endian we can convert the underlying `[u8; 32]`
    /// array to `[u64; 4]`. Since in WebAssembly `u64` is supported natively unlike `u8`
    /// it is more efficient to work on chunks of `u8` represented as `u64`.
    #[cfg(target_endian = "little")]
    fn add_assign_u64_le(&mut self, rhs: u64) {
        let words = self.reinterpret_as_u64x4_mut();
        let mut carry = rhs;
        for word in words {
            let (res, ovfl) = word.overflowing_add(carry);
            *word = res;
            carry = ovfl as u64;
            if carry == 0 {
                return
            }
        }
    }

    /// Adds the `u64` value to the key storing the result in `result`.
    ///
    /// # Note
    ///
    /// This implementation is heavily optimized for little-endian Wasm platforms.
    ///
    /// # Developer Note
    ///
    /// Since we are operating on little-endian we can convert the underlying `[u8; 32]`
    /// array to `[u64; 4]`. Since in WebAssembly `u64` is supported natively unlike `u8`
    /// it is more efficient to work on chunks of `u8` represented as `u64`.
    #[cfg(target_endian = "little")]
    fn add_assign_u64_le_using(&self, rhs: u64, result: &mut Key) {
        let lhs = self.reinterpret_as_u64x4();
        let result = result.reinterpret_as_u64x4_mut();
        let mut carry = rhs;
        for (lhs, result) in lhs.iter().zip(result) {
            let (res, ovfl) = lhs.overflowing_add(carry);
            *result = res;
            carry = ovfl as u64;
            // Note: We cannot bail out early in this case in order to
            //       guarantee that we fully overwrite the result key.
        }
    }

    /// Adds the `u64` value to the `Key`.
    ///
    /// # Note
    ///
    /// This is a fallback implementation that has not been optimized for any
    /// specific target platform or endianess.
    #[cfg(target_endian = "big")]
    fn add_assign_u64_be(&mut self, rhs: u64) {
        let rhs_bytes = rhs.to_be_bytes();
        let lhs_bytes = self.as_mut();
        let len_rhs = rhs_bytes.len();
        let len_lhs = lhs_bytes.len();
        let mut carry = 0;
        for i in 0..len_rhs {
            let (res, ovfl) =
                lhs_bytes[i].overflowing_add(rhs_bytes[i].wrapping_add(carry));
            lhs_bytes[i] = res;
            carry = ovfl as u8;
        }
        for i in len_rhs..len_lhs {
            let (res, ovfl) = lhs_bytes[i].overflowing_add(carry);
            lhs_bytes[i] = res;
            carry = ovfl as u8;
            if carry == 0 {
                return
            }
        }
    }

    /// Adds the `u64` value to the key storing the result in `result`.
    ///
    /// # Note
    ///
    /// This is a fallback implementation that has not been optimized for any
    /// specific target platform or endianess.
    #[cfg(target_endian = "big")]
    fn add_assign_u64_be_using(&self, rhs: u64, result: &mut Key) {
        let rhs_bytes = rhs.to_be_bytes();
        let lhs_bytes = self.as_ref();
        let result_bytes = result.as_mut();
        let len_rhs = rhs_bytes.len();
        let len_lhs = lhs_bytes.len();
        let mut carry = 0;
        for i in 0..len_rhs {
            let (res, ovfl) =
                lhs_bytes[i].overflowing_add(rhs_bytes[i].wrapping_add(carry));
            result_bytes[i] = res;
            carry = ovfl as u8;
        }
        for i in len_rhs..len_lhs {
            let (res, ovfl) = lhs_bytes[i].overflowing_add(carry);
            result_bytes[i] = res;
            carry = ovfl as u8;
            // Note: We cannot bail out early in this case in order to
            //       guarantee that we fully overwrite the result key.
        }
    }

    /// Adds the `u64` value to the key storing the result in `result`.
    ///
    /// # Note
    ///
    /// This will overwrite the contents of the `result` key.
    #[inline]
    pub fn add_assign_u64_using(&self, rhs: u64, result: &mut Key) {
        cfg_if! {
            if #[cfg(target_endian = "little")] {
                self.add_assign_u64_le_using(rhs, result);
            } else {
                self.add_assign_u64_be_using(rhs, result);
            }
        }
    }
}

impl AddAssign<u64> for Key {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        cfg_if! {
            if #[cfg(target_endian = "little")] {
                self.add_assign_u64_le(rhs);
            } else {
                self.add_assign_u64_be(rhs);
            }
        }
    }
}

impl AddAssign<&u64> for Key {
    #[inline]
    fn add_assign(&mut self, rhs: &u64) {
        <Self as AddAssign<u64>>::add_assign(self, *rhs)
    }
}
