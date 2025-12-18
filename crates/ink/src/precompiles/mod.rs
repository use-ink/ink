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

pub mod erc20;

/// Calculates the address of a precompile at index `n`.
///
/// This creates an address with the precompile index encoded in bytes 16-19
/// as big-endian: `0x0000000000000000000000000000000000[nn]0000`.
///
/// # Arguments
/// * `n` - The precompile index (e.g., `0x0120` for ERC20 Assets precompile)
#[inline]
pub fn fixed_address(n: u16) -> ink::Address {
    let shifted = (n as u32) << 16;

    let suffix = shifted.to_be_bytes();
    let mut address = [0u8; 20];
    let mut i = 16;
    while i < address.len() {
        address[i] = suffix[i - 16];
        i += 1;
    }
    ink::Address::from(address)
}

/// Calculates the address of a precompile at index `n` with an additional prefix.
///
/// This is used for precompiles that encode additional information in the address,
/// such as the ERC20 Assets precompile which encodes the asset ID in bytes 0-3.
///
/// The resulting address format is: `[prefix][...00000000000000][nn]0000`
/// where `prefix` occupies bytes 0-3 and `nn` is the precompile index in bytes 16-19.
///
/// # Arguments
/// * `n` - The precompile index (e.g., `0x0120` for ERC20 Assets precompile)
/// * `prefix` - A 32-bit value to encode in the first 4 bytes (e.g., asset ID)
#[inline]
pub fn prefixed_address(n: u16, prefix: u32) -> ink::Address {
    let address = fixed_address(n);
    let mut address_bytes: [u8; 20] = address.into();
    address_bytes[..4].copy_from_slice(&prefix.to_be_bytes());
    ink::Address::from(address_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_address_works() {
        let expected = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00,
        ];
        let result = fixed_address(100);
        let address_bytes: [u8; 20] = result.into();
        assert_eq!(address_bytes, expected);
    }

    #[test]
    fn prefixed_address_works() {
        let expected = [
            0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x65, 0x00, 0x00,
        ];
        let result = prefixed_address(101, u32::MAX);
        let address_bytes: [u8; 20] = result.into();
        assert_eq!(address_bytes, expected);
    }
}
