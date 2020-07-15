// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use super::BinaryHeap;

#[test]
fn new_binary_heap_works() {
    // `BinaryHeap::new`
    let heap = <BinaryHeap<i32>>::new();
    assert!(heap.is_empty());
    assert_eq!(heap.len(), 0);
    assert_eq!(heap.peek(), None);
    assert!(heap.iter().next().is_none());
    // `BinaryHeap::default`
    let default = <BinaryHeap<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert_eq!(heap.peek(), None);
    assert!(default.iter().next().is_none());
    // `BinaryHeap::new` and `BinaryHeap::default` should be equal.
    assert_eq!(heap, default);
}
