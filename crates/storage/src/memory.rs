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

use crate::traits::{
    KeyPtr,
    SpreadLayout,
};
use core::{
    convert::{
        self,
        AsRef,
    },
    fmt,
    fmt::Display,
    ops::{
        Deref,
        DerefMut,
    },
};
use ink_prelude::borrow::{
    Borrow,
    BorrowMut,
};

/// An instance that is solely stored within the contract's memory.
///
/// This will never be stored to or loaded from contract storage.
///
/// # Note
///
/// Use instances of this type in order to have some shared state between
/// contract messages and functions.
/// Its usage is comparable to the Solidity's `memory` instances.
/// Pulling an instance of this type from the contract storage will always
/// yield a default constructed value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Memory<T> {
    /// The inner value that will always be stored within contract memory.
    inner: T,
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        CellLayout,
        Layout,
        LayoutKey,
    };

    impl<T> StorageLayout for Memory<T> {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Cell(CellLayout::new::<()>(LayoutKey::from(
                key_ptr.advance_by(0),
            )))
        }
    }
};

impl<T> SpreadLayout for Memory<T>
where
    T: Default,
{
    const FOOTPRINT: u64 = 0;

    fn pull_spread(_ptr: &mut KeyPtr) -> Self {
        Default::default()
    }

    fn push_spread(&self, _ptr: &mut KeyPtr) {}
    fn clear_spread(&self, _ptr: &mut KeyPtr) {}
}

impl<T> Memory<T> {
    /// Creates a new memory instance.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Returns a shared reference to the inner `T`.
    pub fn get(memory: &Self) -> &T {
        &memory.inner
    }

    /// Returns an exclusive reference to the inner `T`.
    pub fn get_mut(memory: &mut Self) -> &mut T {
        &mut memory.inner
    }
}

impl<T> From<T> for Memory<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T> Default for Memory<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(<T as Default>::default())
    }
}

impl<T> Display for Memory<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        core::fmt::Display::fmt(Self::get(self), f)
    }
}

impl<T> Deref for Memory<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T> DerefMut for Memory<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

impl<T> AsRef<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn as_ref(&self) -> &T {
        Self::get(self)
    }
}

impl<T> convert::AsMut<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn as_mut(&mut self) -> &mut T {
        Self::get_mut(self)
    }
}

impl<T> Borrow<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn borrow(&self) -> &T {
        Self::get(self)
    }
}

impl<T> BorrowMut<T> for Memory<T>
where
    T: SpreadLayout,
{
    fn borrow_mut(&mut self) -> &mut T {
        Self::get_mut(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;
    use crate::traits::{
        KeyPtr,
        SpreadLayout,
    };
    use core::{
        convert::{
            AsMut,
            AsRef,
        },
        ops::{
            Deref,
            DerefMut,
        },
    };
    use ink_env::test::DefaultAccounts;
    use ink_prelude::borrow::{
        Borrow,
        BorrowMut,
    };
    use ink_primitives::Key;

    type ComplexTuple = (u8, [i32; 4], (bool, i32));

    fn complex_value() -> ComplexTuple {
        (b'A', [0x00; 4], (true, 42))
    }

    #[test]
    fn new_works() {
        let mut expected = complex_value();
        let mut mem = Memory::new(expected);
        assert_eq!(<Memory<_> as Deref>::deref(&mem), &expected);
        assert_eq!(<Memory<_> as DerefMut>::deref_mut(&mut mem), &mut expected);
        assert_eq!(<Memory<_> as AsRef<_>>::as_ref(&mem), &expected);
        assert_eq!(<Memory<_> as AsMut<_>>::as_mut(&mut mem), &mut expected);
        assert_eq!(Borrow::<ComplexTuple>::borrow(&mem), &expected);
        assert_eq!(
            BorrowMut::<ComplexTuple>::borrow_mut(&mut mem),
            &mut expected
        );
        assert_eq!(Memory::get(&mem), &expected);
        assert_eq!(Memory::get_mut(&mut mem), &mut expected);
    }

    #[test]
    fn from_works() {
        let mut expected = complex_value();
        let mut from = Memory::from(expected);
        assert_eq!(from, Memory::new(expected));
        assert_eq!(Memory::get(&from), &expected);
        assert_eq!(Memory::get_mut(&mut from), &mut expected);
    }

    #[test]
    fn default_works() {
        use core::fmt::Debug;
        fn assert_default<T>()
        where
            T: Debug + Default + PartialEq,
        {
            let mut memory_default = <Memory<T> as Default>::default();
            let mut default = <T as Default>::default();
            assert_eq!(<Memory<T>>::get(&memory_default), &default);
            assert_eq!(<Memory<T>>::get_mut(&mut memory_default), &mut default);
        }
        assert_default::<bool>();
        assert_default::<u8>();
        assert_default::<Option<i32>>();
        assert_default::<Memory<[u8; 4]>>();
    }

    #[test]
    fn spread_layout_push_pull_works() {
        let p1 = Memory::new((b'A', [0x00; 4], (true, 42)));
        assert_eq!(*p1, (b'A', [0x00; 4], (true, 42)));
        assert_ne!(p1, Default::default());
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&p1, &mut KeyPtr::from(root_key));
        // Now load another instance of a pack from the same key and check
        // if both instances are equal:
        let p2 = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
        assert_ne!(p1, p2);
        assert_eq!(p2, Default::default());
    }

    fn run_test<F>(f: F)
    where
        F: FnOnce(DefaultAccounts<ink_env::DefaultEnvironment>),
    {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|default_accounts| {
            f(default_accounts);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn spread_layout_clear_works() {
        run_test(|_| {
            // Clearing a memory instance should have no effect on the underlying
            // contract storage. We can test this by pushing and pulling a storage
            // affecting entity in between on the same storage region:
            let root_key = Key::from([0x42; 32]);
            <i32 as SpreadLayout>::push_spread(&42, &mut KeyPtr::from(root_key));
            let loaded1 = <i32 as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
            assert_eq!(loaded1, 42);
            let mem = Memory::new(77);
            SpreadLayout::push_spread(&mem, &mut KeyPtr::from(root_key));
            let loaded2 = <i32 as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
            assert_eq!(loaded2, 42);
            // Now we clear the `i32` from storage and check whether that works.
            // We load as `Option<i32>` in order to expect `None`:
            <i32 as SpreadLayout>::clear_spread(&loaded2, &mut KeyPtr::from(root_key));
            use crate::traits::pull_packed_root_opt;
            let loaded3 = pull_packed_root_opt::<Option<i32>>(&root_key);
            assert_eq!(loaded3, None);
        })
    }
}
