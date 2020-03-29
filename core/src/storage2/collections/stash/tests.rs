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

use super::Stash as StorageStash;

#[test]
fn new_works() {
    // `StorageVec::new`
    let stash = <StorageStash<i32>>::new();
    assert!(stash.is_empty());
    assert_eq!(stash.len(), 0);
    assert_eq!(stash.get(0), None);
    assert!(stash.iter().next().is_none());
    // `StorageVec::default`
    let default = <StorageStash<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert_eq!(stash.get(0), None);
    assert!(default.iter().next().is_none());
    // `StorageVec::new` and `StorageVec::default` should be equal.
    assert_eq!(stash, default);
}

#[test]
fn from_iterator_works() {
    let some_primes = [b'A', b'B', b'C', b'D', b'E', b'F'];
    assert_eq!(some_primes.iter().copied().collect::<StorageStash<_>>(), {
        let mut vec = StorageStash::new();
        for (index, prime) in some_primes.iter().enumerate() {
            assert_eq!(index as u32, vec.put(*prime));
        }
        vec
    });
}
