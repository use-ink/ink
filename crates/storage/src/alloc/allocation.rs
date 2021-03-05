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

use ink_env::hash::{
    Blake2x256,
    CryptoHash,
    HashOutput,
};
use ink_primitives::Key;

/// A unique dynamic allocation.
///
/// This can refer to a dynamically allocated storage cell.
/// It has been created by a dynamic storage allocator.
/// The initiator of the allocation has to make sure to deallocate
/// this dynamic allocation again using the same dynamic allocator
/// if it is no longer in use.
///
/// # Note
///
/// Normally instances of this type are not used directly and instead
/// a [`storage::Box`](`crate::Box`) is used instead.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, scale::Encode, scale::Decode,
)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct DynamicAllocation(pub(super) u32);

/// Wraps a bytes buffer and turns it into an accumulator.
///
/// # Panics
///
/// Upon hash calculation if the underlying buffer length does not suffice the
/// needs of the accumulated hash buffer.
struct Wrap<'a> {
    /// The underlying wrapped buffer.
    buffer: &'a mut [u8],
    /// The current length of the filled area.
    len: usize,
}

impl Wrap<'_> {
    /// Returns the capacity of the underlying buffer.
    fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the length of the underlying buffer.
    fn len(&self) -> usize {
        self.len
    }

    /// Appends the given bytes to the end of the wrapped buffer.
    fn append_bytes(&mut self, bytes: &[u8]) {
        debug_assert!(self.len() + bytes.len() <= self.capacity());
        let len = self.len;
        let bytes_len = bytes.len();
        self.buffer[len..(len + bytes_len)].copy_from_slice(bytes);
        self.len += bytes_len;
    }
}

#[cfg(feature = "std")]
impl<'a> std::io::Write for Wrap<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.append_bytes(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> From<&'a mut [u8]> for Wrap<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer, len: 0 }
    }
}

#[cfg(not(feature = "std"))]
impl<'a> scale::Output for Wrap<'a> {
    fn write(&mut self, bytes: &[u8]) {
        self.append_bytes(bytes)
    }

    fn push_byte(&mut self, byte: u8) {
        debug_assert!(self.len() < self.capacity());
        self.buffer[self.len] = byte;
        self.len += 1;
    }
}

impl DynamicAllocation {
    /// Returns the allocation identifier as `u32`.
    pub(super) fn get(self) -> u32 {
        self.0
    }

    /// Returns the storage key associated with this dynamic allocation.
    pub fn key(self) -> Key {
        // We create a 25-bytes buffer for the hashing.
        // This is due to the fact that we prepend the `u32` encoded identifier
        // with the `b"DYNAMICALLY ALLOCATED"` byte string which has a length
        // 21 bytes. Since `u32` always has an encoding length of 4 bytes we
        // end up requiring 25 bytes in total.
        // Optimization Opportunity:
        // Since ink! always runs single threaded we could make this buffer
        // static and instead reuse its contents with every invocation of this
        // method. However, this would introduce `unsafe` Rust usage.
        pub struct EncodeWrapper(u32);
        impl scale::Encode for EncodeWrapper {
            #[rustfmt::skip]
            fn encode_to<O>(&self, output: &mut O)
            where
                O: scale::Output + ?Sized,
            {
                <[u8; 21] as scale::Encode>::encode_to(&[
                    b'D', b'Y', b'N', b'A', b'M', b'I', b'C', b'A', b'L', b'L', b'Y',
                    b' ',
                    b'A', b'L', b'L', b'O', b'C', b'A', b'T', b'E', b'D',
                ], output);
                <u32 as scale::Encode>::encode_to(&self.0, output);
            }
        }
        // Encode the `u32` identifier requires a 4 bytes buffer.
        #[rustfmt::skip]
        let mut buffer: [u8; 25] = [
            b'D', b'Y', b'N', b'A', b'M', b'I', b'C', b'A', b'L', b'L', b'Y',
            b' ',
            b'A', b'L', b'L', b'O', b'C', b'A', b'T', b'E', b'D',
            b'_', b'_', b'_', b'_',
        ];
        {
            let mut wrapped = Wrap::from(&mut buffer[21..25]);
            <u32 as scale::Encode>::encode_to(&self.0, &mut wrapped);
        }
        let mut output = <Blake2x256 as HashOutput>::Type::default();
        <Blake2x256 as CryptoHash>::hash(&buffer, &mut output);
        Key::from(output)
    }
}

#[test]
fn get_works() {
    let expected_keys = [
        b"\
            \x0A\x0F\xF5\x30\xBD\x5A\xB6\x67\
            \x85\xC9\x74\x6D\x01\x33\xD7\xE1\
            \x24\x40\xC4\x67\xA9\xF0\x6D\xCA\
            \xE7\xED\x2E\x78\x32\x77\xE9\x10",
        b"\
            \x11\x5A\xC0\xB2\x29\xA5\x34\x10\
            \xB0\xC0\x2D\x47\x49\xDC\x7A\x09\
            \xB9\x6D\xF9\x51\xB6\x1D\x4F\x3B\
            \x4E\x75\xAC\x3B\x14\x57\x47\x96",
    ];
    assert_eq!(DynamicAllocation(0).key(), Key::from(*expected_keys[0]));
    assert_eq!(DynamicAllocation(1).key(), Key::from(*expected_keys[1]));
}
