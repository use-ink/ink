use crate::storage::{Key, Stash};

const NEXT_VACANT_KEY: Key = Key([0x0; 32]);
const LEN_KEY: Key = Key([0x1; 32]);
const LEN_MAX_KEY: Key = Key([0x2; 32]);
const ENTRIES_KEY: Key = Key([0x3; 32]);

fn empty_stash() -> Stash<i32> {
	unsafe {
		Stash::new_unchecked(
			NEXT_VACANT_KEY,
			LEN_KEY,
			LEN_MAX_KEY,
			ENTRIES_KEY,
		)
	}
}

fn filled_stash() -> Stash<i32> {
	let mut stash = empty_stash();
	stash.put(5);
	stash.put(42);
	stash.put(1337);
	stash.put(77);
	assert_eq!(stash.len(), 4);
	stash
}

#[test]
fn new_unchecked() {
	let stash = empty_stash();
	// Initial invariant.
	assert_eq!(stash.len(), 0);
	assert!(stash.is_empty());
	assert_eq!(stash.iter().next(), None);
}

#[test]
fn put_empty() {
	let mut stash = empty_stash();
	// Before and after first put.
	assert_eq!(stash.get(0), None);
	assert_eq!(stash.put(42), 0);
	assert_eq!(stash.get(0), Some(&42));
}

#[test]
fn put_filled() {
	let mut stash = filled_stash();
	// Before and next put.
	assert_eq!(stash.get(0), Some(&5));
	assert_eq!(stash.get(1), Some(&42));
	assert_eq!(stash.get(2), Some(&1337));
	assert_eq!(stash.get(3), Some(&77));
	assert_eq!(stash.get(4), None);
	assert_eq!(stash.len(), 4);
	// Now put.
	assert_eq!(stash.put(123), 4);
	assert_eq!(stash.get(4), Some(&123));
	assert_eq!(stash.len(), 5);
}

#[test]
fn take_empty() {
	let mut stash = empty_stash();
	assert_eq!(stash.take(0), None);
	assert_eq!(stash.take(1000), None);
}

#[test]
fn take_filled() {
	let mut stash = filled_stash();
	// Take and check len
	assert_eq!(stash.len(), 4);
	assert_eq!(stash.take(0), Some(5));
	assert_eq!(stash.len(), 3);
	assert_eq!(stash.take(1), Some(42));
	assert_eq!(stash.len(), 2);
	assert_eq!(stash.take(2), Some(1337));
	assert_eq!(stash.len(), 1);
	assert_eq!(stash.take(3), Some(77));
	assert_eq!(stash.len(), 0);
	assert_eq!(stash.take(4), None);
	assert_eq!(stash.len(), 0);
}

#[test]
fn put_take() {
	let mut stash = filled_stash();
	// Take and put "randomly"
	//
	// Layout of the stash in memory:
	//
	//          |----------------------------------|
	// Index    |    0 |    1 |    2 |    3 |    4 |
	//          |------|------|------|------|------|
	// Occupied |    5 |   42 | 1337 |   77 |      |
	// Vacant   |      |      |      |      |      |
	//          |----------------------------------|
	// next_vacant = 4
	assert_eq!(stash.take(2), Some(1337));
	//          |----------------------------------|
	// Index    |    0 |    1 |    2 |    3 |    4 |
	//          |------|------|------|------|------|
	// Occupied |    5 |   42 |      |   77 |      |
	// Vacant   |      |      |    4 |      |      |
	//          |----------------------------------|
	// next_vacant = 2
	assert_eq!(stash.take(0), Some(5));
	//          |----------------------------------|
	// Index    |    0 |    1 |    2 |    3 |    4 |
	//          |------|------|------|------|------|
	// Occupied |      |   42 |      |   77 |      |
	// Vacant   |    2 |      |    4 |      |      |
	//          |----------------------------------|
	// next_vacant = 0
	assert_eq!(stash.put(123), 0);
	//          |----------------------------------|
	// Index    |    0 |    1 |    2 |    3 |    4 |
	//          |------|------|------|------|------|
	// Occupied |  123 |   42 |      |   77 |      |
	// Vacant   |      |      |    4 |      |      |
	//          |----------------------------------|
	// next_vacant = 2
	assert_eq!(stash.put(555), 2);
	//          |----------------------------------|
	// Index    |    0 |    1 |    2 |    3 |    4 |
	//          |------|------|------|------|------|
	// Occupied |  123 |   42 |  555 |   77 |      |
	// Vacant   |      |      |      |      |      |
	//          |----------------------------------|
	// next_vacant = 4
	assert_eq!(stash.put(999), 4);
	//          |------------------------------------------
	// Index    |    0 |    1 |    2 |    3 |    4 |    5 |
	//          |------|------|------|------|------|------|
	// Occupied |  123 |   42 |  555 |   77 |  999 |      |
	// Vacant   |      |      |      |      |      |      |
	//          |------------------------------------------
	// next_vacant = 5
	assert_eq!(stash.take(3), Some(77));
	//          |------------------------------------------
	// Index    |    0 |    1 |    2 |    3 |    4 |    5 |
	//          |------|------|------|------|------|------|
	// Occupied |  123 |   42 |  555 |      |  999 |      |
	// Vacant   |      |      |      |    5 |      |      |
	//          |------------------------------------------
	// next_vacant = 3
	assert_eq!(stash.take(0), Some(123));
	//          |------------------------------------------
	// Index    |    0 |    1 |    2 |    3 |    4 |    5 |
	//          |------|------|------|------|------|------|
	// Occupied |      |   42 |  555 |      |  999 |      |
	// Vacant   |    3 |      |      |    5 |      |      |
	//          |------------------------------------------
	// next_vacant = 0
	assert_eq!(stash.put(911), 0);
	//          |------------------------------------------
	// Index    |    0 |    1 |    2 |    3 |    4 |    5 |
	//          |------|------|------|------|------|------|
	// Occupied |  911 |   42 |  555 |      |  999 |      |
	// Vacant   |      |      |      |    5 |      |      |
	//          |------------------------------------------
	// next_vacant = 3
	assert_eq!(stash.take(3), None);
	assert_eq!(stash.take(1), Some(42));
	//          |------------------------------------------
	// Index    |    0 |    1 |    2 |    3 |    4 |    5 |
	//          |------|------|------|------|------|------|
	// Occupied |  911 |      |  555 |      |  999 |      |
	// Vacant   |      |    3 |      |    5 |      |      |
	//          |------------------------------------------
	// next_vacant = 1
}

#[test]
fn iter() {
	let stash = filled_stash();
	let mut iter = stash.iter();
	assert_eq!(iter.next(), Some((0, &5)));
	assert_eq!(iter.next(), Some((1, &42)));
	assert_eq!(iter.next(), Some((2, &1337)));
	assert_eq!(iter.next(), Some((3, &77)));
	assert_eq!(iter.next(), None);
}

fn holey_stash() -> Stash<i32> {
	let mut stash = filled_stash();
	stash.put(123);
	stash.take(1);
	stash.take(3);
	stash
}

#[test]
fn iter_holey() {
	let stash = holey_stash();
	let mut iter = stash.iter();
	assert_eq!(iter.next(), Some((0, &5)));
	assert_eq!(iter.next(), Some((2, &1337)));
	assert_eq!(iter.next(), Some((4, &123)));
	assert_eq!(iter.next(), None);
}

#[test]
fn iter_back() {
	let stash = filled_stash();
	let mut iter = stash.iter();
	assert_eq!(iter.next_back(), Some((3, &77)));
	assert_eq!(iter.next_back(), Some((2, &1337)));
	assert_eq!(iter.next_back(), Some((1, &42)));
	assert_eq!(iter.next_back(), Some((0, &5)));
	assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_back_holey() {
	let stash = holey_stash();
	let mut iter = stash.iter();
	assert_eq!(iter.next_back(), Some((4, &123)));
	assert_eq!(iter.next_back(), Some((2, &1337)));
	assert_eq!(iter.next_back(), Some((0, &5)));
	assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_size_hint() {
	let stash = filled_stash();
	let mut iter = stash.iter();
	assert_eq!(iter.size_hint(), (4, Some(4)));
	iter.next();
	assert_eq!(iter.size_hint(), (3, Some(3)));
}
