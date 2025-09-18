// Copyright (C) Use Ink (UK) Ltd.
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

use ink_primitives::{
    Address,
    H256,
    U256,
    abi::{
        AbiEncodeWith,
        Ink,
        Sol,
    },
    sol::SolResultEncode,
};
use ink_storage_traits::{
    Storable,
    decode_all,
};
use pallet_revive_uapi::{
    CallFlags,
    HostFn,
    HostFnImpl as ext,
    ReturnErrorCode,
    ReturnFlags,
    StorageFlags,
};
#[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
use xcm::VersionedXcm;

use crate::{
    DecodeDispatch,
    DispatchError,
    EnvBackend,
    Environment,
    Result,
    TypedEnvBackend,
    call::{
        Call,
        CallParams,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAddr,
        LimitParamsV2,
        utils::DecodeMessageResult,
    },
    engine::on_chain::{
        EncodeScope,
        EnvInstance,
        ScopedBuffer,
    },
    event::{
        Event,
        TopicEncoder,
        TopicsBuilderBackend,
    },
    hash::{
        Blake2x128,
        Blake2x256,
        CryptoHash,
        HashOutput,
        Keccak256,
        Sha2x256,
    },
    types::FromLittleEndian,
};

impl CryptoHash for Blake2x128 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 16];
        static_assertions::assert_type_eq_all!(
            <Blake2x128 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 16);

        let mut buffer = [0u8; 32];
        let mut output_buffer = [0u8; 32];

        let sel = const { solidity_selector("hashBlake128(bytes)") };
        buffer[..4].copy_from_slice(&sel[..4]);

        let n = solidity_encode_bytes(input, 32, &mut buffer[4..]);

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000900");
        let call_result = ext::delegate_call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &buffer[..4 + n],
            Some(&mut &mut output_buffer[..]),
        );
        call_result.expect("call host function failed");
        output[..].copy_from_slice(&output_buffer[16..]);
    }

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        panic!(
            "Hashing Blake2x128 can be done without a buffer, by calling a host function"
        );
    }
}

/// Returns the Solidity selector for `fn_sig`.
///
/// Note that this is a const function, it is evaluated at compile time.
///
/// # Usage
///
/// ```
/// let sel = solidity_selector("ownCodeHash()");
/// assert_eq!(sel, [219, 107, 220, 138]);
/// ```
pub const fn solidity_selector(fn_sig: &str) -> [u8; 4] {
    let output: [u8; 32] = const_crypto::sha3::Keccak256::new()
        .update(fn_sig.as_bytes())
        .finalize();
    [output[0], output[1], output[2], output[3]]
}

/// Encodes a `u32` to big-endian `[u8; 32]` with padded zeros.
fn encode_u32(value: u32, out: &mut [u8]) {
    debug_assert_eq!(out.len(), 32);
    out[28..].copy_from_slice(&value.to_be_bytes()); // last 4 bytes
}

/// Encodes a `bool` to big-endian `[u8; 32]` with padded zeros.
fn encode_bool(value: bool, out: &mut [u8]) {
    debug_assert_eq!(out.len(), 32);
    if value {
        out[31] = 1;
    }
}

const STORAGE_PRECOMPILE_ADDR: [u8; 20] =
    hex_literal::hex!("0000000000000000000000000000000000000901");

/// Four bytes are required to encode a Solidity selector;
const SOL_ENCODED_SELECTOR_LEN: usize = 4;

/// Number of bytes required to encode the `uint32 flags` parameter
/// that the `Storage` pre-compile functions take.
const SOL_ENCODED_FLAGS_LEN: usize = 32;

/// Number of bytes required to encode the `bool isFixedKey` parameter
/// that the `Storage` pre-compile functions take.
const SOL_ENCODED_IS_FIXED_KEY_LEN: usize = 32;

/// Number of bytes required to store the offset word when encoding
/// to the Solidity type `bytes`.
const SOL_BYTES_OFFSET_WORD_LEN: usize = 32;

/// When encoding a Rust `[u8]` to Solidity `bytes`, a small amount
/// of overhead space is required (for the 32 bytes offset + 32 bytes
/// length word).
const SOL_BYTES_ENCODING_OVERHEAD: usize = 64;

/// Encodes the `bytes` argument for the Solidity ABI.
/// The result is written to `out`.
///
/// Returns the number of bytes written.
///
/// # Developer Note
///
/// The returned layout will be
///
///     `[offset (32 bytes)] [len (32 bytes)] [data (padded to 32)]`
///
/// The `out` byte array need to be able to hold
/// (in the worst case) 95 bytes more than `input.len()`.
///
/// This is because we write the following to `out`:
///   * The offset word → always 32 bytes.
///   * The length word → always 32 bytes.
///   * The input itself → exactly `input.len()` bytes.
///   * We pad the input to a multiple of 32 → between 0 and 31 extra bytes.
fn solidity_encode_bytes(input: &[u8], offset: u32, out: &mut [u8]) -> usize {
    let len = input.len();
    let padded_len = solidity_padded_len(len);

    // out_len = 32 + padded_len
    //         = 32 + ceil(input_len / 32) * 32
    assert!(out.len() >= padded_len + SOL_BYTES_ENCODING_OVERHEAD);

    // Encode offset as a 32-byte big-endian word
    out[28..32].copy_from_slice(&offset.to_be_bytes()[..4]);
    out[..28].copy_from_slice(&[0u8; 28]); // make sure the first bytes are zeroed

    // Encode length as a 32-byte big-endian word
    let mut len_word = [0u8; 32];
    // We are at most on a 64-bit architecture, hence we can safely assume `len < 2^64`.
    let len_bytes = (len as u64).to_be_bytes();
    len_word[24..32].copy_from_slice(&len_bytes);
    out[32..64].copy_from_slice(&len_word);

    // Write data after `offset` and `len` word
    out[64..64 + len].copy_from_slice(input);

    64 + padded_len
}

/// Returns the Solidity word padded length for the given input length (i.e. next multiple
/// of 32 for the given number).
#[inline(always)]
const fn solidity_padded_len(len: usize) -> usize {
    ((len + 31) / 32) * 32
}

impl CryptoHash for Blake2x256 {
    fn hash(_input: &[u8], _output: &mut <Self as HashOutput>::Type) {
        panic!("Hashing Blake2x256 requires calling a pre-compile and a buffer");
    }

    fn hash_with_buffer(
        input: &[u8],
        buffer: &mut [u8],
        output: &mut <Self as HashOutput>::Type,
    ) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Blake2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);

        let sel = const { solidity_selector("hashBlake256(bytes)") };
        buffer[..4].copy_from_slice(&sel[..4]);

        let n = solidity_encode_bytes(input, 32, &mut buffer[4..]);

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000900");
        let call_result = ext::delegate_call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &buffer[..4 + n],
            Some(&mut &mut output[..]),
        );
        call_result.expect("call host function failed");
    }
}

impl CryptoHash for Sha2x256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Sha2x256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000002");
        // todo return value?
        let _ = ext::call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &U256::zero().to_little_endian(), // Value transferred to the contract.
            input,
            Some(&mut &mut output[..]),
        );
    }

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        panic!(
            "Hashing Sha2x256 can be done without a buffer, by calling a host function"
        );
    }
}

impl CryptoHash for Keccak256 {
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type) {
        type OutputType = [u8; 32];
        static_assertions::assert_type_eq_all!(
            <Keccak256 as HashOutput>::Type,
            OutputType
        );
        let output: &mut OutputType = array_mut_ref!(output, 0, 32);
        ext::hash_keccak_256(input, output);
    }

    fn hash_with_buffer(
        _input: &[u8],
        _buffer: &mut [u8],
        _output: &mut <Self as HashOutput>::Type,
    ) {
        panic!(
            "Hashing Keccak256 can be done without a buffer, by calling a host function"
        );
    }
}

pub struct TopicsBuilder<'a> {
    scoped_buffer: ScopedBuffer<'a>,
}

impl<'a> From<ScopedBuffer<'a>> for TopicsBuilder<'a> {
    fn from(scoped_buffer: ScopedBuffer<'a>) -> Self {
        Self { scoped_buffer }
    }
}

impl<'a, Abi> TopicsBuilderBackend<Abi> for TopicsBuilder<'a>
where
    Abi: TopicEncoder,
{
    type Output = (ScopedBuffer<'a>, &'a mut [u8]);

    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: AbiEncodeWith<Abi>,
    {
        let output = if <Abi as TopicEncoder>::REQUIRES_BUFFER {
            let mut output = [0u8; 32];
            let split = self.scoped_buffer.split();
            let buffer = split.take_rest();
            <Abi as TopicEncoder>::encode_topic_with_hash_buffer(
                topic_value,
                &mut output,
                buffer,
            );
            output
        } else {
            <Abi as TopicEncoder>::encode_topic(topic_value)
        };
        self.scoped_buffer.append_bytes(output.as_slice());
    }

    fn output(mut self) -> Self::Output {
        let encoded_topics = self.scoped_buffer.take_appended();
        (self.scoped_buffer, encoded_topics)
    }
}

impl TopicEncoder for Ink {
    const REQUIRES_BUFFER: bool = true;

    fn encode_topic<T>(_value: &T) -> [u8; 32]
    where
        T: AbiEncodeWith<Self>,
    {
        panic!("Blake2x256 hashing requires calling a pre-compile and a buffer");
    }

    fn encode_topic_with_hash_buffer<T>(
        value: &T,
        output: &mut [u8; 32],
        buffer: &mut [u8],
    ) where
        T: AbiEncodeWith<Self>,
    {
        let mut scoped_buffer = ScopedBuffer::from(buffer);

        let encoded = scoped_buffer.take_encoded_with(|buff| value.encode_to_slice(buff));
        let len_encoded = encoded.len();
        let solidity_encoding_buffer = scoped_buffer.take(
            SOL_ENCODED_SELECTOR_LEN
                + SOL_BYTES_ENCODING_OVERHEAD
                + solidity_padded_len(len_encoded),
        );

        if len_encoded <= 32 {
            output[..len_encoded].copy_from_slice(encoded);
        } else {
            <Blake2x256 as CryptoHash>::hash_with_buffer(
                encoded,
                &mut solidity_encoding_buffer[..],
                output,
            );
        }
    }
}

impl TopicEncoder for Sol {
    const REQUIRES_BUFFER: bool = false;

    fn encode_topic<T>(value: &T) -> [u8; 32]
    where
        T: AbiEncodeWith<Self>,
    {
        value.encode_topic(<Keccak256 as CryptoHash>::hash)
    }

    fn encode_topic_with_hash_buffer<T>(
        _value: &T,
        _output: &mut [u8; 32],
        _buffer: &mut [u8],
    ) where
        T: AbiEncodeWith<Self>,
    {
        panic!(
            "Keccak-256 hashing can be done without a buffer, by calling a host function"
        );
    }
}

impl EnvInstance {
    #[inline(always)]
    /// Returns a new scoped buffer for the entire scope of the static 16 kB buffer.
    fn scoped_buffer(&mut self) -> ScopedBuffer<'_> {
        ScopedBuffer::from(&mut self.buffer[..])
    }

    /// Returns the contract property value from its little-endian representation.
    ///
    /// # Note
    ///
    /// This skips the potentially costly decoding step that is often equivalent to a
    /// `memcpy`.
    #[inline(always)]
    fn get_property_little_endian<T>(&mut self, ext_fn: fn(output: &mut [u8; 32])) -> T
    where
        T: FromLittleEndian,
    {
        let mut scope = self.scoped_buffer();
        let u256: &mut [u8; 32] = scope
            .take(32)
            .try_into()
            .expect("failed to take 32 bytes from buffer");
        ext_fn(u256);
        let mut result = <T as FromLittleEndian>::Bytes::default();
        let len = result.as_ref().len();
        result.as_mut()[..].copy_from_slice(&u256[..len]);
        <T as FromLittleEndian>::from_le_bytes(result)
    }
}

fn call_bool_precompile(selector: [u8; 4], output: &mut [u8]) -> bool {
    debug_assert_eq!(output.len(), 32);
    const ADDR: [u8; 20] = hex_literal::hex!("0000000000000000000000000000000000000900");
    let _ = ext::delegate_call(
        CallFlags::empty(),
        &ADDR,
        u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
        u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
        &[u8::MAX; 32], // No deposit limit.
        &selector[..],
        Some(&mut &mut output[..]),
    )
    .expect("call host function failed");
    if output[31] == 1 {
        debug_assert_eq!(&output[..31], [0u8; 31]);
        return true;
    }
    debug_assert_eq!(&output[..32], [0u8; 32]);
    return false;
}

/// Calls a function on the `pallet-revive` `Storage` pre-compile "contract".
///
/// # Developer Note
///
/// This function assumes that the called pre-compiles all have this function
/// signature for the arguments:
///
///     function containsStorage(uint32 flags, bool isFixedKey, bytes memory key)
///
/// The function makes heavy use of operating on byte slices and the positions
/// in the slice are calculated based on the size of these three arguments.
///
/// The return type does not matter.
fn call_storage_precompile(
    input_buf: &mut &mut [u8],
    selector: [u8; 4],
    key: &[u8],
    output: &mut [u8],
) -> core::result::Result<(), ReturnErrorCode> {
    input_buf.fill(0);

    debug_assert_eq!(
        SOL_ENCODED_SELECTOR_LEN
            + SOL_ENCODED_FLAGS_LEN
            + SOL_ENCODED_IS_FIXED_KEY_LEN
            + SOL_BYTES_ENCODING_OVERHEAD
            + solidity_padded_len(key.len()),
        input_buf.len(),
        "input buffer has an unexpected size",
    );

    input_buf[..4].copy_from_slice(&selector[..]);
    encode_u32(STORAGE_FLAGS.bits(), &mut input_buf[4..36]); // todo @cmichi optimize
    encode_bool(false, &mut input_buf[36..68]); // todo @cmichi optimize

    let encoded_bytes_len = solidity_encode_bytes(
        key,
        // The offset is 96 here because all `Storage` pre-compile functions
        // take the input arguments `(uint32 flags, bool isFixedKey, bytes memory key)`.
        //
        // The offset is where the data payload of the third argument, `bytes`, starts:
        // 32 bytes for `flags` + 32 bytes for `isFixedKey` + 32 bytes for the `offset`
        // word that comes first when encoding `bytes`.
        // 96 then points to the `len|data` segment of `bytes`
        (SOL_ENCODED_FLAGS_LEN + SOL_ENCODED_IS_FIXED_KEY_LEN + SOL_BYTES_OFFSET_WORD_LEN)
            as u32,
        // encode the `bytes` starting at the appropriate position in the slice
        &mut input_buf[SOL_ENCODED_SELECTOR_LEN
            + SOL_ENCODED_FLAGS_LEN
            + SOL_ENCODED_IS_FIXED_KEY_LEN..],
    );

    // todo @cmichi check if we might better return `None` in this situation. perhaps a
    // zero sized key is legal?
    debug_assert!(
        encoded_bytes_len >= SOL_BYTES_ENCODING_OVERHEAD + 32,
        "the `bytes` encoding length was < 96, meaning we didn't encode a 32 byte `key`. \
        calling this function without `key` does not make sense and is unexpected."
    );

    // `output` needs to hold at least 32 bytes, for the len field of `bytes`.
    // if no `bytes` are returned from the delegate call we will at the minimum
    // have the `len` field.
    assert!(output.len() >= 32);

    ext::delegate_call(
        CallFlags::empty(),
        &STORAGE_PRECOMPILE_ADDR,
        u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
        u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
        &[u8::MAX; 32], // No deposit limit.
        &input_buf[..SOL_ENCODED_SELECTOR_LEN
            + SOL_ENCODED_FLAGS_LEN
            + SOL_ENCODED_IS_FIXED_KEY_LEN
            + encoded_bytes_len],
        Some(&mut &mut output[..]),
    )
}

const STORAGE_FLAGS: StorageFlags = StorageFlags::empty();

impl EnvBackend for EnvInstance {
    fn set_contract_storage<K, V>(&mut self, key: &K, value: &V) -> Option<u32>
    where
        K: scale::Encode,
        V: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        let value = buffer.take_storable_encoded(value);
        ext::set_storage(STORAGE_FLAGS, key, value)
    }

    fn get_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);
        let output = &mut buffer.take_rest();
        match ext::get_storage(STORAGE_FLAGS, key, output) {
            Ok(_) => (),
            Err(ReturnErrorCode::KeyNotFound) => return Ok(None),
            Err(_) => panic!("encountered unexpected error"),
        }
        let decoded = decode_all(&mut &output[..])?;
        Ok(Some(decoded))
    }

    fn remaining_buffer(&mut self) -> usize {
        self.scoped_buffer().remaining_buffer()
    }

    /// Calls the following function on the `pallet-revive` `Storage` pre-compile:
    ///
    ///     function takeStorage(uint32 flags, bool isFixedKey, bytes memory key)
    ///         external returns (bytes memory)
    fn take_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);

        let padded_len = solidity_padded_len(key.len());
        let buf: &mut [u8] = buffer.take(
            SOL_ENCODED_SELECTOR_LEN
                + SOL_ENCODED_FLAGS_LEN
                + SOL_ENCODED_IS_FIXED_KEY_LEN
                + SOL_BYTES_ENCODING_OVERHEAD
                + padded_len,
        );

        let output = &mut buffer.take_rest();

        let sel = const { solidity_selector("takeStorage(uint32,bool,bytes)") };
        let _ = call_storage_precompile(&mut &mut buf[..], sel, key, output)
            .expect("failed calling Storage pre-compile (take)");

        debug_assert!(
            !output.is_empty(),
            "output must always contain at least the len and offset of `bytes`"
        );

        // extract the `len` from the returned Solidity `bytes`
        let mut buf = [0u8; 4];
        buf[..].copy_from_slice(&output[60..64]);
        let bytes_len = u32::from_be_bytes(buf) as usize;

        if bytes_len == 0 {
            return Ok(None);
        }

        if output.len() < SOL_BYTES_ENCODING_OVERHEAD + bytes_len {
            return Err(crate::Error::BufferTooSmall);
        }

        // We start decoding at the start of the payload.
        // The payload starts at the `len` word here:
        // `bytes = offset (32 bytes) | len (32 bytes) | data`
        let decoded = decode_all(
            &mut &output
                [SOL_BYTES_ENCODING_OVERHEAD..bytes_len + SOL_BYTES_ENCODING_OVERHEAD],
        )?;

        Ok(Some(decoded))
    }

    /// Calls the following function on the `pallet-revive` `Storage` pre-compile:
    ///
    ///	    function containsStorage(uint32 flags, bool isFixedKey, bytes memory key)
    ///     	external returns (bool containedKey, uint valueLen)
    fn contains_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);

        let padded_len = solidity_padded_len(key.len());
        let buf: &mut [u8] = buffer.take(
            SOL_ENCODED_SELECTOR_LEN
                + SOL_ENCODED_FLAGS_LEN
                + SOL_ENCODED_IS_FIXED_KEY_LEN
                + SOL_BYTES_ENCODING_OVERHEAD
                + padded_len,
        );
        let output = buffer.take(64);
        let sel = const { solidity_selector("containsStorage(uint32,bool,bytes)") };
        call_storage_precompile(&mut &mut buf[..], sel, key, &mut &mut output[..])
            .expect("failed calling Storage pre-compile (contains)");

        // Check the returned `containedKey` boolean value
        if output[31] == 0 {
            debug_assert!(
                output.iter().all(|x| *x == 0),
                "both `containedKey` and `valueLen` need to be zero"
            );
            return None;
        }

        let mut value_len_buf = [0u8; 4];
        value_len_buf[..4].copy_from_slice(&output[60..64]);
        Some(u32::from_be_bytes(value_len_buf))
    }

    /// Calls the following function on the `pallet-revive` `Storage` pre-compile:
    ///
    ///     function clearStorage(uint32 flags, bool isFixedKey, bytes memory key)
    ///     	external returns (bool containedKey, uint valueLen);
    fn clear_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode,
    {
        let mut buffer = self.scoped_buffer();
        let key = buffer.take_encoded(key);

        let padded_len = solidity_padded_len(key.len());
        let buf: &mut [u8] = buffer.take(
            SOL_ENCODED_SELECTOR_LEN
                + SOL_ENCODED_FLAGS_LEN
                + SOL_ENCODED_IS_FIXED_KEY_LEN
                + SOL_BYTES_ENCODING_OVERHEAD
                + padded_len,
        );
        let output = buffer.take(64);

        let sel = const { solidity_selector("clearStorage(uint32,bool,bytes)") };
        let _ = call_storage_precompile(&mut &mut buf[..], sel, key, &mut output[..])
            .expect("failed calling Storage pre-compile (clear)");

        // Check the returned `containedKey` boolean value
        if output[31] == 0 {
            debug_assert!(
                output.iter().all(|x| *x == 0),
                "both `containedKey` and `valueLen` need to be zero"
            );
            return None;
        }

        let mut value_len_buf = [0u8; 4];
        value_len_buf[..4].copy_from_slice(&output[60..64]);
        Some(u32::from_be_bytes(value_len_buf))
    }

    fn decode_input<T>(&mut self) -> core::result::Result<T, DispatchError>
    where
        T: DecodeDispatch,
    {
        let full_scope = &mut self.scoped_buffer().take_rest();
        ext::call_data_copy(full_scope, 0);
        DecodeDispatch::decode_dispatch(&mut &full_scope[..])
    }

    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: scale::Encode,
    {
        let mut scope = EncodeScope::from(&mut self.buffer[..]);
        return_value.encode_to(&mut scope);
        let len = scope.len();
        if len == 0 {
            ext::return_value(flags, &[]);
        } else {
            ext::return_value(flags, &self.buffer[..][..len]);
        }
    }

    fn return_value_solidity<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: for<'a> SolResultEncode<'a>,
    {
        let encoded = return_value.encode();
        ext::return_value(flags, &encoded[..]);
    }

    fn hash_bytes<H>(&mut self, input: &[u8], output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
    {
        <H as CryptoHash>::hash(input, output)
    }

    fn hash_encoded<H, T>(&mut self, input: &T, output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
        T: scale::Encode,
    {
        let mut scope = self.scoped_buffer();
        let enc_input = scope.take_encoded(input);
        <H as CryptoHash>::hash(enc_input, output)
    }

    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()> {
        // todo change fn args to just take the slice callee_input slice directly
        let mut callee_input = [0u8; 65 + 32];
        callee_input[..65].copy_from_slice(&signature[..65]);
        callee_input[65..65 + 32].copy_from_slice(&message_hash[..32]);

        const ECRECOVER: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000001");
        // todo return value?
        let _ = ext::call(
            CallFlags::empty(),
            &ECRECOVER,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &U256::zero().to_little_endian(), // Value transferred to the contract.
            &callee_input[..],
            Some(&mut &mut output[..]),
        );
        Ok(())
    }

    #[cfg(feature = "unstable-hostfn")]
    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
    ) -> Result<()> {
        ext::ecdsa_to_eth_address(pubkey, output).map_err(Into::into)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn sr25519_verify(
        &mut self,
        signature: &[u8; 64],
        message: &[u8],
        pub_key: &[u8; 32],
    ) -> Result<()> {
        ext::sr25519_verify(signature, message, pub_key).map_err(Into::into)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn set_code_hash(&mut self, code_hash: &H256) -> Result<()> {
        ext::set_code_hash(code_hash.as_fixed_bytes());
        Ok(()) // todo
    }
}

// TODO remove anything with hash
impl TypedEnvBackend for EnvInstance {
    fn caller(&mut self) -> Address {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::caller(h160);
        // TODO: check decode, I think this could just be from_le_bytes
        scale::Decode::decode(&mut &h160[..])
            .expect("The executed contract must have a caller with a valid account id.")
    }

    fn transferred_value(&mut self) -> U256 {
        let mut scope = self.scoped_buffer();
        let u256: &mut [u8; 32] = scope.take(32).try_into().unwrap();

        ext::value_transferred(u256);
        U256::from_le_bytes(*u256)
    }

    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp {
        self.get_property_little_endian::<E::Timestamp>(ext::now)
    }

    fn account_id<E: Environment>(&mut self) -> E::AccountId {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::address(h160);

        // 32 bytes offset + 32 bytes len + 32 bytes account_id
        let output: &mut [u8; 96] = scope.take(96).try_into().unwrap();

        let selector = const { solidity_selector("toAccountId(address)") };
        let input: &mut [u8; 36] = &mut scope.take(4 + 32).try_into().unwrap();

        input[..4].copy_from_slice(&selector[..]);
        input[16..36].copy_from_slice(&h160[..]);

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000900");
        let _ = ext::delegate_call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &input[..],
            Some(&mut &mut output[..]),
        )
        .expect("call host function failed");

        // We start decoding at the start of the payload.
        // The payload starts at the `len` word here:
        // `bytes = offset (32 bytes) | len (32 bytes) | data`
        scale::Decode::decode(&mut &output[64..96]).expect("must exist")
    }

    #[cfg(feature = "unstable-hostfn")]
    fn to_account_id<E: Environment>(&mut self, addr: Address) -> E::AccountId {
        let mut scope = self.scoped_buffer();
        let account_id: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        ext::to_account_id(addr.as_fixed_bytes(), account_id);
        scale::Decode::decode(&mut &account_id[..])
            .expect("A contract being executed must have a valid account id.")
    }

    fn address(&mut self) -> Address {
        let mut scope = self.scoped_buffer();

        let h160: &mut [u8; 20] = scope.take(20).try_into().unwrap();
        ext::address(h160);

        scale::Decode::decode(&mut &h160[..])
            .expect("A contract being executed must have a valid address.")
    }

    fn balance(&mut self) -> U256 {
        self.get_property_little_endian(ext::balance)
    }

    fn block_number<E: Environment>(&mut self) -> E::BlockNumber {
        self.get_property_little_endian::<E::BlockNumber>(ext::block_number)
    }

    fn minimum_balance(&mut self) -> U256 {
        let selector = const { solidity_selector("minimumBalance()") };
        let output: &mut [u8; 32] = &mut self
            .scoped_buffer()
            .take_max_encoded_len::<U256>()
            .try_into()
            .unwrap();

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000900");
        let _ = ext::delegate_call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &selector[..],
            Some(&mut &mut output[..]),
        )
        .expect("call host function failed");

        U256::from_little_endian(&output[..])
    }

    fn emit_event<Evt, Abi>(&mut self, event: &Evt)
    where
        Evt: Event<Abi>,
        Abi: TopicEncoder,
    {
        let (mut scope, enc_topics) =
            event.topics(TopicsBuilder::from(self.scoped_buffer()).into());
        // TODO: improve
        let enc_topics = enc_topics
            .chunks_exact(32)
            .map(|c| c.try_into().unwrap())
            .collect::<ink_prelude::vec::Vec<[u8; 32]>>();
        let enc_data = scope.take_encoded_with(|buffer| {
            let encoded = event.encode_data();
            let len = encoded.len();
            // NOTE: panics if buffer isn't large enough.
            // This behavior is similar to `ScopedBuffer::take_encoded`.
            buffer[..len].copy_from_slice(&encoded);
            len
        });

        ext::deposit_event(&enc_topics[..], enc_data);
    }

    fn invoke_contract<E, Args, R, Abi>(
        &mut self,
        params: &CallParams<E, Call, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: AbiEncodeWith<Abi>,
        R: DecodeMessageResult<Abi>,
    {
        let mut scope = self.scoped_buffer();
        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let storage_deposit_limit = params.storage_deposit_limit();

        let enc_storage_limit = to_u256(&mut scope, storage_deposit_limit);

        let enc_callee: &[u8; 20] = params.callee().as_ref().try_into().unwrap();
        let mut enc_transferred_value = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&params.transferred_value(), &mut enc_transferred_value);
        let enc_transferred_value: &mut [u8; 32] =
            enc_transferred_value.into_buffer().try_into().unwrap();
        let call_flags = params.call_flags();
        let enc_input = if !call_flags.contains(CallFlags::FORWARD_INPUT)
            && !call_flags.contains(CallFlags::CLONE_INPUT)
        {
            scope.take_encoded_with(|buffer| params.exec_input().encode_to_slice(buffer))
        } else {
            &mut []
        };
        let output = &mut scope.take_rest();
        let flags = params.call_flags();
        let call_result = ext::call(
            *flags,
            enc_callee,
            ref_time_limit,
            proof_size_limit,
            &enc_storage_limit,
            enc_transferred_value,
            enc_input,
            Some(output),
        );
        match call_result {
            Ok(()) => R::decode_output(output, false),
            Err(ReturnErrorCode::CalleeReverted) => R::decode_output(output, true),
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    fn invoke_contract_delegate<E, Args, R, Abi>(
        &mut self,
        params: &CallParams<E, DelegateCall, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: AbiEncodeWith<Abi>,
        R: DecodeMessageResult<Abi>,
    {
        let mut scope = self.scoped_buffer();
        let call_flags = params.call_flags();
        let enc_input = if !call_flags.contains(CallFlags::FORWARD_INPUT)
            && !call_flags.contains(CallFlags::CLONE_INPUT)
        {
            scope.take_encoded_with(|buffer| params.exec_input().encode_to_slice(buffer))
        } else {
            &mut []
        };
        let deposit_limit = params.deposit_limit();
        let deposit_limit = remove_option(&mut scope, *deposit_limit);

        let output = &mut scope.take_rest();
        let flags = params.call_flags();
        let enc_address: [u8; 20] = params.address().0;
        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let call_result = ext::delegate_call(
            *flags,
            &enc_address,
            ref_time_limit,
            proof_size_limit,
            &deposit_limit,
            enc_input,
            Some(output),
        );
        match call_result {
            Ok(()) => R::decode_output(output, false),
            Err(ReturnErrorCode::CalleeReverted) => R::decode_output(output, true),
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    fn instantiate_contract<E, ContractRef, Args, RetType, Abi>(
        &mut self,
        params: &CreateParams<E, ContractRef, LimitParamsV2, Args, RetType, Abi>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <RetType as ConstructorReturnType<ContractRef, Abi>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAddr,
        Args: AbiEncodeWith<Abi>,
        RetType: ConstructorReturnType<ContractRef, Abi>,
    {
        let mut scoped = self.scoped_buffer();

        let ref_time_limit = params.ref_time_limit();
        let proof_size_limit = params.proof_size_limit();
        let storage_deposit_limit = params.storage_deposit_limit().map(|limit| {
            let mut enc_storage_limit = EncodeScope::from(scoped.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        });
        let enc_storage_limit = remove_option(&mut scoped, storage_deposit_limit);

        // todo encodings here are mostly unnecessary, as the type is already 32 bytes
        let enc_code_hash: &mut [u8; 32] = scoped
            .take_encoded(params.code_hash())
            .try_into()
            .expect("unable to take 32 for code_hash");
        let mut enc_endowment = EncodeScope::from(scoped.take(32));
        scale::Encode::encode_to(&params.endowment(), &mut enc_endowment);
        let enc_endowment: &mut [u8; 32] =
            enc_endowment.into_buffer().try_into().unwrap();
        let enc_input = scoped
            .take_encoded_with(|buffer| params.exec_input().encode_to_slice(buffer));
        let mut out_address: [u8; 20] =
            scoped.take(20).try_into().expect("unable to take 20");
        let salt = params.salt_bytes().as_ref();

        let input_and_code_hash = scoped.take(32 + enc_input.len());
        input_and_code_hash[..32].copy_from_slice(enc_code_hash);
        input_and_code_hash[32..].copy_from_slice(enc_input);

        let mut output_data = &mut scoped.take_rest();

        let instantiate_result = ext::instantiate(
            ref_time_limit,
            proof_size_limit,
            &enc_storage_limit,
            enc_endowment,
            input_and_code_hash,
            Some(&mut out_address),
            Some(&mut output_data),
            salt,
        );

        crate::engine::decode_instantiate_result::<_, ContractRef, RetType, Abi>(
            instantiate_result.map_err(Into::into),
            &mut &out_address[..],
            &output_data[..],
        )
    }

    #[cfg(feature = "unstable-hostfn")]
    fn terminate_contract(&mut self, beneficiary: Address) -> ! {
        let buffer: &mut [u8; 20] = self.scoped_buffer().take_encoded(&beneficiary)
            [0..20]
            .as_mut()
            .try_into()
            .unwrap();
        ext::terminate(buffer);
    }

    fn transfer<E>(&mut self, destination: Address, value: U256) -> Result<()>
    where
        E: Environment,
    {
        let mut scope = self.scoped_buffer();
        let enc_callee: &[u8; 20] = destination.as_ref().try_into().unwrap();
        let mut enc_value = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&value, &mut enc_value);
        let enc_value: &mut [u8; 32] = enc_value.into_buffer().try_into().unwrap();

        let mut enc_limit = EncodeScope::from(scope.take(32));
        scale::Encode::encode_to(&U256::MAX, &mut enc_limit);
        let enc_limit: &mut [u8; 32] = enc_limit.into_buffer().try_into().unwrap();

        let output = &mut scope.take_rest();
        #[allow(deprecated)]
        let call_result = ext::call(
            CallFlags::empty(),
            enc_callee,
            u64::MAX,
            u64::MAX,
            enc_limit,
            enc_value,
            &[],
            Some(output),
        );
        match call_result {
            Ok(()) => {
                // TODO: clean comments?
                // no need to decode, is ()
                Ok(())
            }
            Err(actual_error) => Err(actual_error.into()),
        }
    }

    fn weight_to_fee<E: Environment>(&mut self, gas: u64) -> E::Balance {
        let mut scope = self.scoped_buffer();
        let u256: &mut [u8; 32] = scope.take(32).try_into().unwrap();
        // TODO: needs ref and proof
        ext::weight_to_fee(gas, gas, u256);
        let mut result = <E::Balance as FromLittleEndian>::Bytes::default();
        let len = result.as_ref().len();
        result.as_mut().copy_from_slice(&u256[..len]);
        <E::Balance as FromLittleEndian>::from_le_bytes(result)
    }

    #[cfg(feature = "unstable-hostfn")]
    fn is_contract(&mut self, _addr: &Address) -> bool {
        panic!(
            "todo call code() precompile, see https://github.com/paritytech/polkadot-sdk/pull/9001"
        );
    }

    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment,
    {
        let sel = const { solidity_selector("callerIsOrigin()") };
        let output: &mut [u8; 32] =
            &mut self.scoped_buffer().take(32).try_into().unwrap();
        call_bool_precompile(sel, output)
    }

    fn caller_is_root<E>(&mut self) -> bool
    where
        E: Environment,
    {
        let sel = const { solidity_selector("callerIsRoot()") };
        let output: &mut [u8; 32] =
            &mut self.scoped_buffer().take(32).try_into().unwrap();
        call_bool_precompile(sel, output)
    }

    fn code_hash(&mut self, addr: &Address) -> Result<H256> {
        let mut scope = self.scoped_buffer();
        // todo can be simplified
        let enc_addr: &mut [u8; 20] =
            scope.take_encoded(addr)[..20].as_mut().try_into().unwrap();
        let output: &mut [u8; 32] =
            scope.take_max_encoded_len::<H256>().try_into().unwrap();
        ext::code_hash(enc_addr, output);
        let hash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }

    fn own_code_hash(&mut self) -> Result<H256> {
        let sel = const { solidity_selector("ownCodeHash()") };
        let output: &mut [u8; 32] = &mut self
            .scoped_buffer()
            .take_max_encoded_len::<H256>()
            .try_into()
            .unwrap();

        const ADDR: [u8; 20] =
            hex_literal::hex!("0000000000000000000000000000000000000900");
        let call_result = ext::delegate_call(
            CallFlags::empty(),
            &ADDR,
            u64::MAX,       // `ref_time` to devote for execution. `u64::MAX` = all
            u64::MAX,       // `proof_size` to devote for execution. `u64::MAX` = all
            &[u8::MAX; 32], // No deposit limit.
            &sel[..],
            Some(&mut &mut output[..]),
        );
        call_result.expect("call host function failed");
        let hash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
    }

    #[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
    fn xcm_execute<E, Call>(&mut self, _msg: &VersionedXcm<Call>) -> Result<()>
    where
        E: Environment,
        Call: scale::Encode,
    {
        panic!(
            "todo Native ink! XCM functions are not supported yet, you have to call the pre-compile contracts for XCM directly until then."
        );
        /*
        let mut scope = self.scoped_buffer();

        let enc_msg = scope.take_encoded(msg);

        #[allow(deprecated)]
        ext::xcm_execute(enc_msg).map_err(Into::into)
        */
    }

    #[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
    // todo
    fn xcm_send<E, Call>(
        &mut self,
        _dest: &xcm::VersionedLocation,
        _msg: &VersionedXcm<Call>,
    ) -> Result<xcm::v4::XcmHash>
    where
        E: Environment,
        Call: scale::Encode,
    {
        panic!(
            "todo Native ink! XCM functions are not supported yet, you have to call the pre-compile contracts for XCM directly until then."
        );
        /*
        let mut scope = self.scoped_buffer();
        let output = scope.take(32);
        scope.append_encoded(dest);
        let enc_dest = scope.take_appended();

        scope.append_encoded(msg);
        let enc_msg = scope.take_appended();
        #[allow(deprecated)]
        ext::xcm_send(enc_dest, enc_msg, output.try_into().unwrap())?;
        let hash: xcm::v4::XcmHash = scale::Decode::decode(&mut &output[..])?;
        Ok(hash)
        */
    }
}

// todo make this const
fn to_u256(scope: &mut ScopedBuffer, value: Option<U256>) -> [u8; 32] {
    let limit = match value {
        None => U256::MAX,
        Some(u256) => u256,
    };
    let mut enc_storage_limit = EncodeScope::from(scope.take(32));
    scale::Encode::encode_to(&limit, &mut enc_storage_limit);
    let enc_storage_limit: [u8; 32] = enc_storage_limit.into_buffer().try_into().unwrap();
    enc_storage_limit
}

fn remove_option(scope: &mut ScopedBuffer, opt: Option<[u8; 32]>) -> [u8; 32] {
    match opt {
        None => {
            let limit = U256::MAX; // corresponds to no deposit limit, defined in `pallet-revive`
            let mut enc_storage_limit = EncodeScope::from(scope.take(32));
            scale::Encode::encode_to(&limit, &mut enc_storage_limit);
            let enc_storage_limit: [u8; 32] =
                enc_storage_limit.into_buffer().try_into().unwrap();
            enc_storage_limit
        }
        Some(bytes) => bytes,
    }
}
