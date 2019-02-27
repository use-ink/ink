// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    storage::{
        self,
        alloc::{
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
        Key,
    },
    test_utils::run_test,
};

fn new_empty<K, V>() -> storage::HashMap<K, V> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        storage::HashMap::allocate_using(&mut alloc).initialize_into(())
    }
}

#[test]
fn new_unchecked() {
    run_test(|| {
        let map = new_empty::<u8, String>();
        assert_eq!(map.len(), 0);
        assert_eq!(map.is_empty(), true);
    })
}

#[test]
fn get() {
    run_test(|| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Black".into(), "White".into()), None);
        assert_eq!(map.insert("Up".into(), "Down".into()), None);
        // Check if get returns the right answer
        assert_eq!(map.get("Black"), Some(&"White".into()));
        assert_eq!(map.get("Up"), Some(&"Down".into()));
        assert_eq!(map.get("Forward"), None);
    })
}

#[test]
fn index() {
    run_test(|| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Black".into(), "White".into()), None);
        assert_eq!(map.insert("Up".into(), "Down".into()), None);
        // Check if get returns the right answer
        assert_eq!(map["Black"], "White");
        assert_eq!(map["Up"], "Down");
    })
}

#[test]
fn index_repeat() {
    run_test(|| {
        let mut map = new_empty::<String, String>();
        // Inserts some elements
        assert_eq!(map.insert("Something".into(), "There it is!".into()), None);
        // Check if get returns the right answer repeatedly
        assert_eq!(map["Something"], "There it is!");
        assert_eq!(map["Something"], "There it is!");
    })
}

#[test]
#[should_panic]
fn index_fail0() {
    let map = new_empty::<String, String>();
    // This will just fail and panic
    &map["Won't catch this!"];
}

#[test]
fn insert() {
    run_test(|| {
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
    })
}

#[test]
fn contains() {
    run_test(|| {
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
    })
}

#[test]
fn remove() {
    run_test(|| {
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
    })
}

#[test]
fn mutate_with() {
    run_test(|| {
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
    })
}
