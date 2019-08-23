// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

/// Types that are able to flush their state into the contract storage.
///
/// # Note
///
/// Many types support caching of their state into memory to avoid costly
/// contract storage reads or writes. When execution of a contract is finished
/// or interrupted (e.g. due to calling a remote contract) we have to flush
/// all cached state into the contract storage.
///
/// # Implementation Hints
///
/// Caching types provided by ink! are `SyncCell` for caching of a single data
/// and `SyncChunk` for caching an array of data.
///
/// All abstractions built upon them that do not have their own caching mechanism
/// shall simply forward flushing to their interiors. Examples for this are
/// `storage::Vec` or `storage::Value`.
pub trait Flush {
    /// Flushes the cached state back to the contract storage, if any.
    ///
    /// # Note
    ///
    /// Needs to take `self` by `&mut` since `SyncChunk` and `SyncCell`
    /// and potentially other abstraction facilities are required to
    /// write back their cached values which is a mutable operation.
    fn flush(&mut self);
}

macro_rules! impl_empty_flush_for {
	( $($ty:ty),* ) => {
		$(
			impl Flush for $ty {
				fn flush(&mut self) {}
			}
		)*
	};
}

impl_empty_flush_for! {
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    bool, char, str
}

macro_rules! impl_tuple_flush_for {
	( $(($n:tt, $name:ident)),* ) => {
		impl< $($name),* > Flush for ($($name,)*)
		where
			$(
				$name: Flush,
			)*
		{
			fn flush(&mut self) {
				$(
					self.$n.flush();
				)*
			}
		}
	}
}

impl_tuple_flush_for!();
impl_tuple_flush_for!((0, A));
impl_tuple_flush_for!((0, A), (1, B));
impl_tuple_flush_for!((0, A), (1, B), (2, C));
impl_tuple_flush_for!((0, A), (1, B), (2, C), (3, D));
impl_tuple_flush_for!((0, A), (1, B), (2, C), (3, D), (4, E));
impl_tuple_flush_for!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
impl_tuple_flush_for!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
impl_tuple_flush_for!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H)
);

macro_rules! impl_array_flush_for {
	( $($n:literal),* ) => {
		$(
			impl<T> Flush for [T; $n]
			where
				T: Flush,
			{
				fn flush(&mut self) {
					for elem in &mut self[..] {
						elem.flush()
					}
				}
			}
		)*
	}
}

#[rustfmt::skip]
impl_array_flush_for!(
	 0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
	10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
	20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
	30, 31, 32
);

impl<T> Flush for [T]
where
    T: Flush,
{
    fn flush(&mut self) {
        for elem in self {
            elem.flush()
        }
    }
}

impl<T> Flush for Option<T>
where
    T: Flush,
{
    fn flush(&mut self) {
        match self {
            Some(val) => val.flush(),
            None => (),
        }
    }
}

impl<T, E> Flush for Result<T, E>
where
    T: Flush,
    E: Flush,
{
    fn flush(&mut self) {
        match self {
            Ok(val) => val.flush(),
            Err(err) => err.flush(),
        }
    }
}

impl<T> Flush for crate::memory::vec::Vec<T>
where
    T: Flush,
{
    fn flush(&mut self) {
        for elem in self {
            elem.flush()
        }
    }
}

impl Flush for crate::memory::string::String {
    fn flush(&mut self) {
        // Note: Strings contain only characters that need no flushing.
    }
}

impl<K, V> Flush for crate::memory::collections::btree_map::BTreeMap<K, V>
where
    V: Flush,
{
    fn flush(&mut self) {
        for (_key, val) in self {
            // We do not need to write back keys since they are immutable.
            val.flush();
        }
    }
}

impl<T> Flush for crate::memory::collections::btree_set::BTreeSet<T> {
    fn flush(&mut self) {
        // Note: Values within a `BTreeSet` are immutable and thus need not be flushed.
    }
}

impl<T> Flush for crate::memory::collections::linked_list::LinkedList<T>
where
    T: Flush,
{
    fn flush(&mut self) {
        for elem in self {
            elem.flush()
        }
    }
}

impl<T> Flush for crate::memory::collections::vec_deque::VecDeque<T>
where
    T: Flush,
{
    fn flush(&mut self) {
        for elem in self {
            elem.flush()
        }
    }
}

impl<T> Flush for crate::memory::collections::binary_heap::BinaryHeap<T>
where
    T: Flush,
{
    fn flush(&mut self) {
        // Note: Values within a `BinaryHeap` are immutable and thus need not be flushed.
    }
}
