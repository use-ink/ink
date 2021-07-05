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

use crate::ext::{
    Engine,
    Error,
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
    let output = &mut &mut get_buffer()[..];
    let res = engine.get_storage(key, output);
    assert_eq!(res, Err(Error::KeyNotFound));

    engine.set_storage(key, &[0x05_u8; 5]);
    let res = engine.get_storage(key, output);
    assert_eq!(res, Ok(()),);
    assert_eq!(output[..5], [0x05; 5]);

    engine.clear_storage(key);
    let res = engine.get_storage(key, output);
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
        .expect("decoding balance failed");
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
        event.topics.get(0).expect("first topic must exist"),
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
#[should_panic(
    expected = "the output buffer is too small! the decoded storage is of size 16 bytes, but the output buffer has only room for 8."
)]
fn must_panic_when_buffer_too_small() {
    // given
    let mut engine = Engine::new();
    engine.set_callee(vec![1; 32]);
    let key: &[u8; 32] = &[0x42; 32];
    engine.set_storage(key, &[0x05_u8; 16]);

    // when
    let mut small_buffer = [0; 8];
    let output = &mut &mut small_buffer[..];
    let _ = engine.get_storage(key, output);

    // then
    unreachable!("`get_storage` must already have panicked");
}
