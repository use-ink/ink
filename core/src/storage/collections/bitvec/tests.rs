use crate::{
	storage::{
		self,
		Key,
		alloc::{
			BumpAlloc,
			AllocateUsing,
			Initialize,
		},
	},
};

/// Returns an empty storage bit vector.
fn new_empty_bitvec() -> storage::BitVec {
	unsafe {
		let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
		storage::BitVec::allocate_using(&mut alloc)
			.initialize_into(())
	}
}

/// Iterator over the bits that are contained in the filled test bit vector.
fn filled_bits() -> impl Iterator<Item = bool> + DoubleEndedIterator {
	vec![true, false, false, true, true, false].into_iter()
}

/// Returns a storage bit vector that is initialized as `100110`.
fn new_filled_bitvec() -> storage::BitVec {
	let mut bv = new_empty_bitvec();
	for bit in filled_bits() {
		bv.push(bit)
	}
	assert_eq!(bv.len(), 6);
	bv
}

#[test]
fn is_empty() {
	let mut bv = new_empty_bitvec();
	assert_eq!(bv.is_empty(), true);
	bv.push(true);
	assert_eq!(bv.is_empty(), false);
	bv.pop();
	assert_eq!(bv.is_empty(), true);
}

#[test]
fn len() {
	const N: u32 = 5;
	let mut bv = new_empty_bitvec();
	for n in 1..=N {
		bv.push(false);
		assert_eq!(bv.len(), n);
	}
	for n in 1..=N {
		bv.pop();
		assert_eq!(bv.len(), N - n);
	}
}

#[test]
fn first() {
	assert_eq!(new_filled_bitvec().first(), Some(true))
}

#[test]
fn first_when_empty() {
	assert_eq!(new_empty_bitvec().first(), None)
}

#[test]
fn last() {
	assert_eq!(new_filled_bitvec().last(), Some(false))
}

#[test]
fn last_when_empty() {
	assert_eq!(new_empty_bitvec().last(), None)
}

#[test]
fn get_filled() {
	let filled = new_filled_bitvec(); // `100110`
	for (n, bit) in filled_bits().enumerate() {
		assert_eq!(filled.get(n as u32), Some(bit));
	}
	assert_eq!(filled.get(6), None);
}

#[test]
fn push_empty() {
	let mut empty = new_empty_bitvec();
	empty.push(true);
	assert_eq!(empty.last(), Some(true));
	assert_eq!(empty.len(), 1);
}

#[test]
fn push_filled() {
	let mut filled = new_filled_bitvec();
	let len = filled.len();
	filled.push(false);
	assert_eq!(filled.last(), Some(false));
	assert_eq!(filled.len(), len + 1);
}

#[test]
fn pop_empty() {
	assert_eq!(new_empty_bitvec().pop(), None);
}

#[test]
fn pop_filled() {
	let mut filled = new_filled_bitvec();
	for bit in filled_bits().rev() {
		assert_eq!(filled.pop(), Some(bit));
	}
	assert_eq!(filled.pop(), None);
}

#[test]
#[should_panic]
fn set_empty() {
	new_empty_bitvec().set(0, true)
}

#[test]
fn set_filled() {
	let mut filled = new_filled_bitvec();
	for (n, bit) in filled_bits().rev().enumerate() {
		filled.set(n as u32, bit);
		assert_eq!(filled.get(n as u32), Some(bit));
	}
}

#[test]
#[should_panic]
fn set_out_of_bounds() {
	let mut filled = new_filled_bitvec();
	let len = filled.len();
	filled.set(len, true);
}

#[test]
#[should_panic]
fn flip_empty() {
	new_empty_bitvec().flip(0)
}

#[test]
#[should_panic]
fn flip_out_of_bounds() {
	let mut filled = new_filled_bitvec();
	let len = filled.len();
	filled.flip(len);
}

#[test]
fn flip_filled() {
	let mut filled = new_filled_bitvec();
	for (n, bit) in filled_bits().enumerate() {
		filled.flip(n as u32);
		assert_eq!(filled.get(n as u32), Some(!bit));
	}
}

#[test]
fn iter() {
	let filled = new_filled_bitvec();
	for (actual, expected) in filled.iter().zip(filled_bits()) {
		assert_eq!(actual, expected)
	}
}
