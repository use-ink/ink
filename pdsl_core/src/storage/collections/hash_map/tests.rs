use crate::storage::{
	self,
	Key,
};

#[test]
fn new_unchecked() {
	let map = unsafe {
		storage::HashMap::<u8, String>::new_unchecked(Key([0x77; 32]))
	};
	assert_eq!(map.len(), 0);
	assert_eq!(map.is_empty(), true);
}

#[test]
fn insert() {
	let mut map = unsafe {
		storage::HashMap::<String, String>::new_unchecked(Key([0x77; 32]))
	};
	assert_eq!(map.len(), 0);
	// Insert empty
	assert_eq!(map.insert("1".into(), "Hello".into()), None);
	// Should increase len.
	assert_eq!(map.len(), 1);
	// Insert overwrite
	assert_eq!(map.insert("1".into(), ", World!".into()), Some("Hello".into()));
	// Should *not* increase len.
	assert_eq!(map.len(), 1);
	// Should return the new value
	assert_eq!(map.get("1"), Some(&", World!".into()));
}

#[test]
fn contains() {
	let mut map = unsafe {
		storage::HashMap::<String, String>::new_unchecked(Key([0x77; 32]))
	};
	// Inserts some elements
	assert_eq!(map.insert("x".into(), "Anton".into()), None);
	assert_eq!(map.insert("y".into(), "Wolfram".into()), None);
	// Check contains `true`
	assert_eq!(map.contains_key("x"), true);
	assert_eq!(map.contains_key("y"), true);
	// Check contains `false`
	assert_eq!(map.contains_key("Anton"), false);
	assert_eq!(map.contains_key(""), false);
}

#[test]
fn remove() {
	let mut map = unsafe {
		storage::HashMap::<String, String>::new_unchecked(Key([0x77; 32]))
	};
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
}

#[test]
fn mutate_with() {
	let mut map = unsafe {
		storage::HashMap::<String, String>::new_unchecked(Key([0x77; 32]))
	};
	// Inserts some elements
	assert_eq!(map.insert("Dog Breed".into(), "Akita".into()), None); // Shiba Inu
	assert_eq!(map.insert("Cat Breed".into(), "Bengal".into()), None); // Burmilla
	// Verify the inserted breeds
	assert_eq!(map.get("Dog Breed"), Some(&"Akita".into()));
	assert_eq!(map.get("Cat Breed"), Some(&"Bengal".into()));
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
	assert_eq!(map.get("Dog Breed"), Some(&"Shiba Inu".into()));
	assert_eq!(map.get("Cat Breed"), Some(&"Bengal Shorthair".into()));
	// Mutate for non-existing key
	assert_eq!(map.mutate_with("Bird Breed", |breed| *breed = "Parrot".into()), None);
}
