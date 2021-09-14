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

use ink_eth_compatibility::{
    ECDSAPublicKey,
    EthereumAddress,
};

#[test]
fn correct_to_eth_address() {
    #[rustfmt::skip]
    let pub_key: ECDSAPublicKey = ECDSAPublicKey {
        0: [
            2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160,  98, 149, 206, 135, 11,
            7,   2, 155, 252, 219,  45, 206,  40, 217, 89, 242, 129,  91,  22, 248, 23,
            152,
        ]
    };

    #[rustfmt::skip]
    const EXPECTED_ETH_ADDRESS: EthereumAddress = [
        126, 95, 69, 82, 9, 26, 105, 18, 93, 93, 252, 183, 184, 194, 101, 144, 41, 57, 91, 223
    ];

    assert_eq!(pub_key.to_eth_address(), EXPECTED_ETH_ADDRESS);
}

#[test]
fn correct_to_account_id() {
    #[rustfmt::skip]
    let pub_key: ECDSAPublicKey = ECDSAPublicKey {
        0: [
            2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160,  98, 149, 206, 135, 11,
            7,   2, 155, 252, 219,  45, 206,  40, 217, 89, 242, 129,  91,  22, 248, 23,
            152,
        ]
    };

    #[rustfmt::skip]
    const EXPECTED_ACCOUNT_ID: [u8; 32] = [
        41, 117, 241, 210, 139, 146, 182, 232,  68, 153, 184, 59,   7, 151, 239, 82,
        53,  85,  62, 235, 126, 218, 160, 206, 162,  67, 193, 18, 140,  47, 231, 55,
    ];

    assert_eq!(pub_key.to_account_id(), EXPECTED_ACCOUNT_ID.into());
}
