// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use super::SmallVec;
use generic_array::typenum::*;

#[test]
fn new_vec_works() {
    let vec = <SmallVec<i32, U4>>::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.get(0), None);
    assert!(vec.iter().next().is_none());
    let default = <SmallVec<i32, U4> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert_eq!(vec.get(0), None);
    assert!(default.iter().next().is_none());
}

#[test]
fn from_iterator_works() {
    let some_primes = [b'A', b'B', b'C', b'D'];
    assert_eq!(some_primes.iter().copied().collect::<SmallVec<_, U4>>(), {
        let mut vec = SmallVec::new();
        for prime in &some_primes {
            vec.push(*prime)
        }
        vec
    });
}

#[test]
#[should_panic]
fn from_iterator_too_many() {
    let some_primes = [b'A', b'B', b'C', b'D', b'E'];
    let _ = some_primes.iter().copied().collect::<SmallVec<_, U4>>();
}

#[test]
fn from_empty_iterator_works() {
    assert_eq!(
        [].iter().copied().collect::<SmallVec<u8, U4>>(),
        SmallVec::new(),
    );
}

#[test]
fn first_last_of_empty() {
    let mut vec = <SmallVec<u8, U4>>::new();
    assert_eq!(vec.first(), None);
    assert_eq!(vec.first_mut(), None);
    assert_eq!(vec.last(), None);
    assert_eq!(vec.last_mut(), None);
}

#[test]
fn pop_on_empty_works() {
    let mut vec = <SmallVec<u8, U4>>::new();
    assert_eq!(vec.pop(), None);
}
