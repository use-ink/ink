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
    clear_spread_root,
    forward_clear_packed,
    forward_pull_packed,
    forward_push_packed,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_prelude::vec::Vec;
use ink_primitives::Key;

/// Packs the inner `T` so that it only occupies a single contract storage cell.
///
/// # Note
///
/// This is an important modular building stone in order to manage contract
/// storage occupation. By default, types try to distribute themselves onto
/// their respective contract storage area. However, upon packing them into
/// `Pack<T>` they will be compressed to only ever make use of a single
/// contract storage cell. Sometimes this can be advantageous for performance
/// reasons.
///
/// # Usage
///
/// - A `Pack<i32>` instance is equivalent to `i32` in its storage occupation.
/// - A `Pack<(i32, i32)>` instance will occupy a single cell compared to
///   `(i32, i32)` which occupies a cell per `i32`.
/// - A `Lazy<Pack<[u8; 8]>>` lazily loads a `Pack<[u8; 8]>` which occupies
///   a single cell whereas a `[u8; 8]` array would occupy 8 cells in total,
///   one for each `u8`.
/// - Rust collections will never use more than a single cell. So
///   `Pack<LinkedList<T>>` and `LinkedList<T>` will occupy the same amount of
///   cells, namely 1.
/// - Packs can be packed. So for example a
///   `Pack<(Pack<(i32, i32)>, Pack<[u8; 8]>)` uses just one cell instead of
///   two cells which is the case for `(Pack<(i32, i32)>, Pack<[u8; 8]>)`.
/// - Not all `storage` types can be packed. Only those that are implementing
///   the `PackedLayout` trait. For example `storage::Vec<T>` does not implement
///   this trait and thus cannot be packed.
///
/// As a general advice pack values together that are frequently used together.
/// Also pack many very small elements (e.g. `u8`, `bool`, `u16`) together.
#[derive(Debug, Clone)]
pub struct Pack<T>
where
    T: PackedLayout,
{
    /// The packed `T` value.
    inner: T,
    /// The key to load the packed value from.
    ///
    /// # Note
    ///
    /// This can be `None` on contract initialization, but will be
    /// initialized with a concrete value on `pull_spread`.
    key: Option<Key>,
}

impl<T> scale::Encode for Pack<T>
where
    T: scale::Encode + PackedLayout,
{
    #[inline]
    fn size_hint(&self) -> usize {
        <T as scale::Encode>::size_hint(&self.inner)
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, dest: &mut O) {
        <T as scale::Encode>::encode_to(&self.inner, dest)
    }

    #[inline]
    fn encode(&self) -> Vec<u8> {
        <T as scale::Encode>::encode(&self.inner)
    }

    #[inline]
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        <T as scale::Encode>::using_encoded(&self.inner, f)
    }
}

impl<T> scale::Decode for Pack<T>
where
    T: scale::Decode + PackedLayout,
{
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        Ok(Self::new(<T as scale::Decode>::decode(input)?))
    }
}

impl<T> Pack<T>
where
    T: PackedLayout,
{
    /// Creates a new packed value.
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            key: None,
        }
    }

    /// Returns a shared reference to the packed value.
    pub fn as_inner(pack: &Pack<T>) -> &T {
        &pack.inner
    }

    /// Returns an exclusive reference to the packed value.
    pub fn as_inner_mut(pack: &mut Pack<T>) -> &mut T {
        &mut pack.inner
    }
}

impl<T> Drop for Pack<T>
where
    T: PackedLayout,
{
    fn drop(&mut self) {
        if let Some(key) = self.key {
            clear_spread_root::<T>(&self.inner, &key)
        }
    }
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        CellLayout,
        Layout,
        LayoutKey,
    };
    use scale_info::TypeInfo;

    impl<T> StorageLayout for Pack<T>
    where
        T: PackedLayout + TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Cell(CellLayout::new::<T>(LayoutKey::from(key_ptr.advance_by(1))))
        }
    }
};

impl<T> SpreadLayout for Pack<T>
where
    T: PackedLayout,
{
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        let inner = forward_pull_packed::<T>(ptr);
        Self {
            inner,
            key: Some(*ptr.key()),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<T>(Self::as_inner(self), ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<T>(Self::as_inner(self), ptr)
    }
}

impl<T> PackedLayout for Pack<T>
where
    T: PackedLayout,
{
    fn pull_packed(&mut self, at: &Key) {
        <T as PackedLayout>::pull_packed(Self::as_inner_mut(self), at)
    }
    fn push_packed(&self, at: &Key) {
        <T as PackedLayout>::push_packed(Self::as_inner(self), at)
    }
    fn clear_packed(&self, at: &Key) {
        <T as PackedLayout>::clear_packed(Self::as_inner(self), at)
    }
}

impl<T> From<T> for Pack<T>
where
    T: PackedLayout,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for Pack<T>
where
    T: Default + PackedLayout,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> core::ops::Deref for Pack<T>
where
    T: PackedLayout,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::as_inner(self)
    }
}

impl<T> core::ops::DerefMut for Pack<T>
where
    T: PackedLayout,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::as_inner_mut(self)
    }
}

impl<T> core::cmp::PartialEq for Pack<T>
where
    T: PartialEq + PackedLayout,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(Self::as_inner(self), Self::as_inner(other))
    }
}

impl<T> core::cmp::Eq for Pack<T> where T: Eq + PackedLayout {}

impl<T> core::cmp::PartialOrd for Pack<T>
where
    T: PartialOrd + PackedLayout,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(Self::as_inner(self), Self::as_inner(other))
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(Self::as_inner(self), Self::as_inner(other))
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(Self::as_inner(self), Self::as_inner(other))
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(Self::as_inner(self), Self::as_inner(other))
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(Self::as_inner(self), Self::as_inner(other))
    }
}

impl<T> core::cmp::Ord for Pack<T>
where
    T: core::cmp::Ord + PackedLayout,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(Self::as_inner(self), Self::as_inner(other))
    }
}

impl<T> core::fmt::Display for Pack<T>
where
    T: core::fmt::Display + PackedLayout,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(Self::as_inner(self), f)
    }
}

impl<T> core::hash::Hash for Pack<T>
where
    T: core::hash::Hash + PackedLayout,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Self::as_inner(self).hash(state);
    }
}

impl<T> core::convert::AsRef<T> for Pack<T>
where
    T: PackedLayout,
{
    fn as_ref(&self) -> &T {
        Self::as_inner(self)
    }
}

impl<T> core::convert::AsMut<T> for Pack<T>
where
    T: PackedLayout,
{
    fn as_mut(&mut self) -> &mut T {
        Self::as_inner_mut(self)
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for Pack<T>
where
    T: PackedLayout,
{
    fn borrow(&self) -> &T {
        Self::as_inner(self)
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for Pack<T>
where
    T: PackedLayout,
{
    fn borrow_mut(&mut self) -> &mut T {
        Self::as_inner_mut(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Pack;
    use crate::traits::{
        pull_packed_root,
        push_packed_root,
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    };
    use core::{
        cmp::Ordering,
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
        let mut pack = Pack::new(expected);
        assert_eq!(<Pack<_> as Deref>::deref(&pack), &expected);
        assert_eq!(<Pack<_> as DerefMut>::deref_mut(&mut pack), &mut expected);
        assert_eq!(<Pack<_> as AsRef<_>>::as_ref(&pack), &expected);
        assert_eq!(<Pack<_> as AsMut<_>>::as_mut(&mut pack), &mut expected);
        assert_eq!(Borrow::<ComplexTuple>::borrow(&pack), &expected);
        assert_eq!(
            BorrowMut::<ComplexTuple>::borrow_mut(&mut pack),
            &mut expected
        );
        assert_eq!(Pack::as_inner(&pack), &expected);
        assert_eq!(Pack::as_inner_mut(&mut pack), &mut expected);
        assert_eq!(pack.inner, expected);
    }

    #[test]
    fn from_works() {
        let mut expected = complex_value();
        let mut from = Pack::from(expected);
        assert_eq!(from, Pack::new(expected));
        assert_eq!(Pack::as_inner(&from), &expected);
        assert_eq!(Pack::as_inner_mut(&mut from), &mut expected);
        assert_eq!(from.inner, expected);
    }

    #[test]
    fn default_works() {
        use core::fmt::Debug;
        fn assert_default<T>()
        where
            T: Debug + Default + PartialEq + PackedLayout,
        {
            let pack_default = <Pack<T> as Default>::default();
            assert_eq!(pack_default.inner, <T as Default>::default());
        }
        assert_default::<bool>();
        assert_default::<u8>();
        assert_default::<Option<i32>>();
        assert_default::<Pack<[u8; 4]>>();
    }

    #[test]
    fn partial_eq_works() {
        let b1 = Pack::new(b'X');
        let b2 = Pack::new(b'Y');
        let b3 = Pack::new(b'X');
        assert!(<Pack<u8> as PartialEq>::ne(&b1, &b2));
        assert!(<Pack<u8> as PartialEq>::eq(&b1, &b3));
    }

    #[test]
    fn partial_ord_works() {
        let b1 = Pack::new(1);
        let b2 = Pack::new(2);
        let b3 = Pack::new(1);
        assert_eq!(
            <Pack<u8> as PartialOrd>::partial_cmp(&b1, &b2),
            Some(Ordering::Less)
        );
        assert_eq!(
            <Pack<u8> as PartialOrd>::partial_cmp(&b2, &b1),
            Some(Ordering::Greater)
        );
        assert_eq!(
            <Pack<u8> as PartialOrd>::partial_cmp(&b1, &b3),
            Some(Ordering::Equal)
        );
        // Less-than
        assert!(<Pack<u8> as PartialOrd>::lt(&b1, &b2));
        // Less-than-or-equals
        assert!(<Pack<u8> as PartialOrd>::le(&b1, &b2));
        assert!(<Pack<u8> as PartialOrd>::le(&b1, &b3));
        // Greater-than
        assert!(<Pack<u8> as PartialOrd>::gt(&b2, &b1));
        // Greater-than-or-equals
        assert!(<Pack<u8> as PartialOrd>::ge(&b2, &b1));
        assert!(<Pack<u8> as PartialOrd>::ge(&b3, &b1));
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
    fn spread_layout_push_pull_works() {
        run_test(|_| {
            let p1 = Pack::new((b'A', [0x00; 4], (true, 42)));
            assert_eq!(*p1, (b'A', [0x00; 4], (true, 42)));
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&p1, &mut KeyPtr::from(root_key));
            // Now load another instance of a pack from the same key and check
            // if both instances are equal:
            let p2 = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
            assert_eq!(p1, p2);
        })
    }

    #[test]
    #[should_panic(expected = "storage entry was empty")]
    fn spread_layout_clear_works() {
        run_test(|_| {
            let p1 = Pack::new((b'A', [0x00; 4], (true, 42)));
            assert_eq!(*p1, (b'A', [0x00; 4], (true, 42)));
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&p1, &mut KeyPtr::from(root_key));
            // Now load another instance of a pack from the same key and check
            // if both instances are equal:
            let p2 = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
            assert_eq!(p1, p2);
            // Clearing the underlying storage of p2 immediately so that
            // loading another instance of pack again should panic.
            SpreadLayout::clear_spread(&p2, &mut KeyPtr::from(root_key));
            let p3 = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));
            assert_eq!(p1, p3);
        })
    }

    #[test]
    fn spread_and_packed_layout_are_equal() {
        run_test(|_| {
            // Push as spread, pull as packed:
            let p1 = Pack::new((b'A', [0x00; 4], (true, 42)));
            assert_eq!(*p1, (b'A', [0x00; 4], (true, 42)));
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&p1, &mut KeyPtr::from(root_key));
            let p2 = pull_packed_root::<Pack<(u8, [i32; 4], (bool, i32))>>(&root_key);
            assert_eq!(p1, p2);
            // Push as packed, pull as spread:
            let root_key2 = Key::from([0x43; 32]);
            push_packed_root(&p2, &root_key2);
            let p3 = SpreadLayout::pull_spread(&mut KeyPtr::from(root_key2));
            assert_eq!(p2, p3);
        })
    }
}

#[cfg(all(test, feature = "std", feature = "ink-fuzz-tests"))]
use quickcheck::{
    Arbitrary,
    Gen,
};

#[cfg(all(test, feature = "std", feature = "ink-fuzz-tests"))]
impl<T: Arbitrary + PackedLayout + Send + Clone + 'static> Arbitrary for Pack<T> {
    fn arbitrary(g: &mut Gen) -> Pack<T> {
        let a = <T as Arbitrary>::arbitrary(g);
        Pack::new(a)
    }
}
