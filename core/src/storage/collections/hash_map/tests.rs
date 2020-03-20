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

use crate::{
    env,
    env::Result,
    storage::{
        self,
        alloc::{
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
    },
};
use ink_primitives::Key;

fn new_empty<K, V>() -> storage::HashMap<K, V> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        storage::HashMap::allocate_using(&mut alloc).initialize_into(())
    }
}

#[test]
fn new_unchecked() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let map = new_empty::<u8, String>();
        assert_eq!(map.len(), 0);
        assert_eq!(map.is_empty(), true);
        Ok(())
    })
}

#[test]
fn get() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Black".into(), "White".into()), None);
        assert_eq!(map.insert("Up".into(), "Down".into()), None);
        // Check if get returns the right answer
        assert_eq!(map.get("Black"), Some(&"White".into()));
        assert_eq!(map.get("Up"), Some(&"Down".into()));
        assert_eq!(map.get("Forward"), None);
        Ok(())
    })
}

#[test]
fn index() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Black".into(), "White".into()), None);
        assert_eq!(map.insert("Up".into(), "Down".into()), None);
        // Check if get returns the right answer
        assert_eq!(map["Black"], "White");
        assert_eq!(map["Up"], "Down");
        Ok(())
    })
}

#[test]
fn index_repeat() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Something".into(), "There it is!".into()), None);
        // Check if get returns the right answer repeatedly
        assert_eq!(map["Something"], "There it is!");
        assert_eq!(map["Something"], "There it is!");
        Ok(())
    })
}

#[test]
#[should_panic]
fn index_fail0() {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let map = new_empty::<String, String>();
        // This will just fail and panic
        let _ = &map["Won't catch this!"];
        Ok(())
    })
    .unwrap()
}

#[test]
fn insert() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        assert_eq!(map.len(), 0);
        // Insert empty
        assert_eq!(map.insert("1".into(), "Hello".into()), None);
        // Should increase len.
        assert_eq!(map.len(), 1);
        // Insert overwrite
        assert_eq!(
            map.insert("1".into(), ", World!".into()),
            Some("Hello".into())
        );
        // Should *not* increase len.
        assert_eq!(map.len(), 1);
        // Should return the new value
        assert_eq!(map["1"], ", World!");
        Ok(())
    })
}

#[test]
fn contains() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("x".into(), "Anton".into()), None);
        assert_eq!(map.insert("y".into(), "Wolfram".into()), None);
        // Check contains `true`
        assert_eq!(map.contains_key("x"), true);
        assert_eq!(map.contains_key("y"), true);
        // Check contains `false`
        assert_eq!(map.contains_key("Anton"), false);
        assert_eq!(map.contains_key(""), false);
        Ok(())
    })
}

#[test]
fn remove() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Dog".into(), "Animal".into()), None);
        assert_eq!(map.insert("Ant".into(), "Insect".into()), None);
        assert_eq!(map.insert("Eel".into(), "Fish".into()), None);
        assert_eq!(map.insert("Robin".into(), "Bird".into()), None);
        assert_eq!(map.len(), 4);
        // Remove them again
        assert_eq!(map.remove("Eel"), Some("Fish".into()));
        assert_eq!(map.len(), 3);
        assert_eq!(map.remove("Ant"), Some("Insect".into()));
        assert_eq!(map.len(), 2);
        assert_eq!(map.remove("Dog"), Some("Animal".into()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.remove("Robin"), Some("Bird".into()));
        assert_eq!(map.len(), 0);
        Ok(())
    })
}

#[test]
fn mutate_with() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Dog Breed".into(), "Akita".into()), None);
        assert_eq!(map.insert("Cat Breed".into(), "Bengal".into()), None);
        assert_eq!(map["Dog Breed"], "Akita");
        assert_eq!(map["Cat Breed"], "Bengal");
        // Change the breeds
        assert_eq!(
            map.mutate_with("Dog Breed", |breed| *breed = "Shiba Inu".into()),
            Some(&"Shiba Inu".into())
        );
        assert_eq!(
            map.mutate_with("Cat Breed", |breed| breed.push_str(" Shorthair")),
            Some(&"Bengal Shorthair".into())
        );
        // Verify the mutated breeds
        assert_eq!(map["Dog Breed"], "Shiba Inu");
        assert_eq!(map["Cat Breed"], "Bengal Shorthair");
        // Mutate for non-existing key
        assert_eq!(
            map.mutate_with("Bird Breed", |breed| *breed = "Parrot".into()),
            None
        );
        Ok(())
    })
}

#[test]
fn extend_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let arr1 = [(1i32, 2i32), (3, 4), (5, 6)];
        let arr2 = [(7i32, 8i32), (9, 10)];
        let mut map = new_empty::<i32, i32>();

        let mut expected = ink_prelude::collections::HashMap::new();

        expected.extend(arr1.iter().cloned());
        expected.extend(arr2.iter().cloned());

        // when
        map.extend(arr1.iter());
        map.extend(arr2.iter());

        // then
        assert_eq!(map.len() as usize, expected.len());
        for (k, v) in &expected {
            assert_eq!(Some(v), map.get(k))
        }
        Ok(())
    })
}
