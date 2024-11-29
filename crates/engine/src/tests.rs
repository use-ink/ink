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

use crate::ext::{
    Engine,
    Error,
};
use secp256k1::{
    ecdsa::RecoverableSignature,
    Message,
    PublicKey,
    SecretKey,
    SECP256K1,
};

/// The public methods of the `contracts` pallet write their result into an
/// `output` buffer instead of returning them. Since we aim to emulate this
/// behavior, we have to provide some buffer for our tests to pass into these
/// emulated methods, so that they can write their result into it.
///
/// The number 1024 is more or less arbitrary, it just satisfies the need of
/// our tests without being too large.
fn get_buffer() -> [u8; 1024] {
    [0; 1024]
}

#[test]
fn store_load_clear() {
    let mut engine = Engine::new();
    engine.set_callee(vec![1; 32]);
    let key: &[u8; 32] = &[0x42; 32];
    let res = engine.get_storage(key);
    assert_eq!(res, Err(Error::KeyNotFound));

    engine.set_storage(key, &[0x05_u8; 5]);
    let res = engine.get_storage(key);
    assert!(res.is_ok());
    assert_eq!(res.unwrap()[..5], [0x05; 5]);

    engine.clear_storage(key);
    let res = engine.get_storage(key);
    assert_eq!(res, Err(Error::KeyNotFound));
}

#[test]
fn setting_getting_balance() {
    // given
    let mut engine = Engine::new();
    let account_id = vec![1; 32];
    let balance = 1337;
    engine.set_callee(account_id.clone());
    engine.set_balance(account_id, balance);

    // when
    let mut output = get_buffer();
    engine.balance(&mut &mut output[..]);

    // then
    let output = <u128 as scale::Decode>::decode(&mut &output[..16])
        .unwrap_or_else(|err| panic!("decoding balance failed: {err}"));
    assert_eq!(output, balance);
}

#[test]
fn setting_getting_caller() {
    // given
    let mut engine = Engine::new();
    let account_id = vec![1; 32];

    // when
    engine.set_caller(account_id.clone());

    // then
    let mut output = get_buffer();
    engine.caller(&mut &mut output[..]);
    assert_eq!(&output[..account_id.len()], &account_id);
}

#[test]
fn address() {
    // given
    let mut engine = Engine::new();
    let account_id = vec![1; 32];
    engine.set_callee(account_id.clone());

    // when
    let mut output = get_buffer();
    engine.address(&mut &mut output[..]);

    // then
    assert_eq!(&output[..account_id.len()], &account_id);
}

#[test]
fn transfer() {
    // given
    let mut engine = Engine::new();
    let alice = vec![1; 32];
    let bob = vec![2; 32];
    engine.set_callee(alice.clone());
    engine.set_balance(alice.clone(), 1337);

    // when
    let val = scale::Encode::encode(&337u128);
    assert_eq!(engine.transfer(&bob, &val), Ok(()));

    // then
    assert_eq!(engine.get_balance(alice), Ok(1000));
    assert_eq!(engine.get_balance(bob), Ok(337));
}

#[test]
fn debug_messages() {
    let mut engine = Engine::new();
    engine.debug_message("foobar");
    let mut recorded = engine.get_emitted_debug_messages().into_iter();
    assert_eq!(recorded.next(), Some("foobar".into()));
    assert_eq!(recorded.next(), None);
}

#[test]
fn events() {
    // given
    let mut engine = Engine::new();
    let topics_count: scale::Compact<u32> = scale::Compact(2u32);
    let mut enc_topics_count = scale::Encode::encode(&topics_count);
    let topic1 = vec![12u8, 13];
    let topic2 = vec![14u8, 15];
    let data = &vec![21, 22, 23];

    // when
    let mut enc_topics_info: Vec<u8> = Vec::new();
    enc_topics_info.append(&mut enc_topics_count);
    enc_topics_info.append(&mut topic1.clone());
    enc_topics_info.append(&mut topic2.clone());
    engine.deposit_event(&enc_topics_info, data);

    // then
    let mut events = engine.get_emitted_events();
    let event = events.next().expect("event must exist");
    assert_eq!(event.topics.len(), 2);
    assert_eq!(
        event.topics.first().expect("first topic must exist"),
        &topic1
    );
    assert_eq!(
        event.topics.get(1).expect("second topic must exist"),
        &topic2
    );
    assert_eq!(&event.data, data);
    assert!(events.next().is_none());
}

#[test]
fn value_transferred() {
    // given
    let mut engine = Engine::new();
    let value = 1337;
    engine.set_value_transferred(value);

    // when
    let output = &mut &mut get_buffer()[..];
    engine.value_transferred(output);

    // then
    let output = <u128 as scale::Decode>::decode(&mut &output[..16])
        .expect("decoding value transferred failed");
    assert_eq!(output, value);
}

#[test]
fn ecdsa_recovery_test_from_contracts_pallet() {
    // given
    let mut engine = Engine::new();
    #[rustfmt::skip]
    let signature: [u8; 65] = [
        161, 234, 203,  74, 147, 96,  51, 212,   5, 174, 231,   9, 142,  48, 137, 201,
        162, 118, 192,  67, 239, 16,  71, 216, 125,  86, 167, 139,  70,   7,  86, 241,
         33,  87, 154, 251,  81, 29, 160,   4, 176, 239,  88, 211, 244, 232, 232,  52,
        211, 234, 100, 115, 230, 47,  80,  44, 152, 166,  62,  50,   8,  13,  86, 175,
         28,
    ];
    #[rustfmt::skip]
    let message_hash: [u8; 32] = [
        162, 28, 244, 179, 96, 76, 244, 178, 188,  83, 230, 248, 143, 106,  77, 117,
        239, 95, 244, 171, 65, 95,  62, 153, 174, 166, 182,  28, 130,  73, 196, 208
    ];

    // when
    let mut output = [0; 33];
    engine
        .ecdsa_recover(&signature, &message_hash, &mut output)
        .expect("must work");

    // then
    #[rustfmt::skip]
    const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
          2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160,  98, 149, 206, 135, 11,
          7,   2, 155, 252, 219,  45, 206,  40, 217, 89, 242, 129,  91,  22, 248, 23,
        152,
    ];
    assert_eq!(output, EXPECTED_COMPRESSED_PUBLIC_KEY);
}

#[test]
fn ecdsa_recovery_with_secp256k1_crate() {
    // given
    let mut engine = Engine::new();
    let seckey = [
        59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107, 94, 203,
        174, 253, 102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28,
    ];
    let pubkey = PublicKey::from_slice(&[
        2, 29, 21, 35, 7, 198, 183, 43, 14, 208, 65, 139, 14, 112, 205, 128, 231, 245,
        41, 91, 141, 134, 245, 114, 45, 63, 82, 19, 251, 210, 57, 79, 54,
    ])
    .expect("pubkey creation failed");

    let mut msg_hash = [0; 32];
    crate::hashing::sha2_256(b"Some message", &mut msg_hash);

    let msg = Message::from_digest_slice(&msg_hash).expect("message creation failed");
    let seckey = SecretKey::from_slice(&seckey).expect("secret key creation failed");
    let recoverable_signature: RecoverableSignature =
        SECP256K1.sign_ecdsa_recoverable(&msg, &seckey);

    let recovery_id = recoverable_signature.serialize_compact().0.to_i32() as u8;
    let mut signature = recoverable_signature.serialize_compact().1.to_vec();
    signature.push(recovery_id);
    let signature_with_recovery_id: [u8; 65] = signature
        .try_into()
        .expect("unable to create signature with recovery id");

    // when
    let mut output = [0; 33];
    engine
        .ecdsa_recover(&signature_with_recovery_id, msg.as_ref(), &mut output)
        .expect("ecdsa recovery failed");

    // then
    assert_eq!(output, pubkey.serialize());
}

#[test]
fn setting_getting_block_timestamp() {
    // given
    let mut engine = Engine::new();
    let new_block_timestamp: u64 = 1000;
    let output = &mut &mut get_buffer()[..];

    // when
    engine.advance_block();
    engine.set_block_timestamp(new_block_timestamp);
    engine.block_timestamp(output);

    // then
    let output = <u64 as scale::Decode>::decode(&mut &output[..16])
        .expect("decoding value transferred failed");
    assert_eq!(output, new_block_timestamp);
}

#[test]
fn setting_getting_block_number() {
    // given
    let mut engine = Engine::new();
    let new_block_number: u32 = 1000;
    let output = &mut &mut get_buffer()[..];

    // when
    engine.advance_block();
    engine.set_block_number(new_block_number);
    engine.block_number(output);

    // then
    let output = <u32 as scale::Decode>::decode(&mut &output[..16])
        .expect("decoding value transferred failed");
    assert_eq!(output, new_block_number);
}
