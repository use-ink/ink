// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use core::fmt::Write;
use derive_more::From;
use ink_prelude::collections::btree_map::BTreeMap;
use ink_primitives::{
    Key,
    KeyPtr,
};
use type_metadata::{
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    IntoCompact,
    Metadata,
    Registry,
};

/// Implemented by types that have a storage layout.
pub trait StorageLayout {
    /// Returns the static storage layout of `Self`.
    ///
    /// The given key pointer is guiding the allocation of static fields onto
    /// the contract storage regions.
    fn layout(key_ptr: &mut KeyPtr) -> Layout;
}

/// Represents the static storage layout of an ink! smart contract.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, From, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub enum Layout<F: Form = MetaForm> {
    /// A layout that can potentially hit the entire storage key space.
    ///
    /// This is commonly used by ink! hashmaps and similar data structures.
    Unbounded(UnboundedLayout<F>),
    /// An array of associated storage cells encoded with a given type.
    ///
    /// This can also represent only a single cell.
    Array(ArrayLayout<F>),
    /// A struct layout with fields of different types.
    Struct(StructLayout<F>),
    /// An enum layout with a discriminant telling which variant is layed out.
    Enum(EnumLayout<F>),
}

impl IntoCompact for Layout {
    type Output = Layout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        match self {
            Layout::Unbounded(unbounded_layout) => {
                Layout::Unbounded(unbounded_layout.into_compact(registry))
            }
            Layout::Array(array_layout) => {
                Layout::Array(array_layout.into_compact(registry))
            }
            Layout::Struct(struct_layout) => {
                Layout::Struct(struct_layout.into_compact(registry))
            }
            Layout::Enum(enum_layout) => Layout::Enum(enum_layout.into_compact(registry)),
        }
    }
}

/// An unbounded layout potentially hitting all cells of the storage.
///
/// Every unbounded layout has an offset and a strategy to compute their keys.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct UnboundedLayout<F: Form = MetaForm> {
    /// The encoded type of the unbounded layout.
    ty: <F as Form>::TypeId,
    /// The key offset used by the strategy.
    offset: LayoutKey,
    /// The actual strategy to compute the unbounded keys.
    strategy: UnboundedStrategy,
}

impl UnboundedLayout {
    /// Creates a new unbounded layout.
    pub fn new<T, K, S>(offset: K, strategy: S) -> Self
    where
        T: Metadata,
        K: Into<LayoutKey>,
        S: Into<UnboundedStrategy>,
    {
        Self {
            ty: <T as Metadata>::meta_type(),
            offset: offset.into(),
            strategy: strategy.into(),
        }
    }
}

impl IntoCompact for UnboundedLayout {
    type Output = UnboundedLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        UnboundedLayout {
            ty: registry.register_type(&self.ty),
            offset: self.offset,
            strategy: self.strategy,
        }
    }
}

/// One of the supported unbounded strategies.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum UnboundedStrategy {
    /// The strategy using a built-in crypto hasher for the computation.
    Hashing(UnboundedHashingStrategy),
}

/// The unbounded hashing strategy.
///
/// The offset key is used as another postfix for the computation.
/// So the actual formula is: `hasher(prefix + encoded(key) + offset + postfix)`
/// Where `+` in this contexts means append of the byte slices.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct UnboundedHashingStrategy {
    /// One of the supported crypto hashers.
    hasher: CryptoHasher,
    /// An optional prefix to the computed hash.
    prefix: Vec<u8>,
    /// An optional postfix to the computed hash.
    postfix: Vec<u8>,
}

impl UnboundedHashingStrategy {
    /// Creates a new unbounded hashing strategy.
    pub fn new(hasher: CryptoHasher, prefix: Vec<u8>, postfix: Vec<u8>) -> Self {
        Self {
            hasher,
            prefix,
            postfix,
        }
    }
}

/// One of the supported crypto hashers.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum CryptoHasher {
    /// The BLAKE-2 crypto hasher with an output of 256 bits.
    Blake2x256,
    /// The SHA-2 crypto hasher with an output of 256 bits.
    Sha2x256,
    /// The KECCAK crypto hasher with an output of 256 bits.
    Keccak256,
}

/// A layout for an array of associated cells with the same encoding.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct ArrayLayout<F: Form = MetaForm> {
    ty: <F as Form>::TypeId,
    offset: LayoutKey,
    len: u32,
}

/// A pointer into some storage region.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutKey {
    key: [u8; 32],
}

impl serde::Serialize for LayoutKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.key;
        let mut hex = String::with_capacity(bytes.len() * 2 + 2);
        write!(hex, "0x").expect("failed writing to string");
        for byte in &bytes {
            write!(hex, "{:02x}", byte).expect("failed writing to string");
        }
        serializer.serialize_str(&hex)
    }
}

impl From<Key> for LayoutKey {
    fn from(key: Key) -> Self {
        let mut arr = [0x00; 32];
        arr.copy_from_slice(key.as_bytes());
        Self { key: arr }
    }
}

impl ArrayLayout {
    /// Creates an array layout for a single storage cell.
    pub fn single<T, K>(at: K) -> Self
    where
        T: Metadata,
        K: Into<LayoutKey>,
    {
        Self {
            ty: <T as Metadata>::meta_type(),
            offset: at.into(),
            len: 1,
        }
    }

    /// Creates an array layout with the given length.
    pub fn new<T, K>(at: K, len: u32) -> Self
    where
        T: Metadata,
        K: Into<LayoutKey>,
    {
        Self {
            ty: <T as Metadata>::meta_type(),
            offset: at.into(),
            len,
        }
    }
}

impl IntoCompact for ArrayLayout {
    type Output = ArrayLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        ArrayLayout {
            offset: self.offset,
            len: self.len,
            ty: registry.register_type(&self.ty),
        }
    }
}

/// A struct layout with consecutive fields of different layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct StructLayout<F: Form = MetaForm> {
    /// The fields of the struct layout.
    fields: Vec<FieldLayout<F>>,
}

impl StructLayout {
    /// Creates a new struct layout.
    pub fn new<F>(fields: F) -> Self
    where
        F: IntoIterator<Item = FieldLayout>,
    {
        Self {
            fields: fields.into_iter().collect(),
        }
    }
}

impl IntoCompact for StructLayout {
    type Output = StructLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        StructLayout {
            fields: self
                .fields
                .into_iter()
                .map(|field| field.into_compact(registry))
                .collect::<Vec<_>>(),
        }
    }
}

/// The layout for a particular field of a struct layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct FieldLayout<F: Form = MetaForm> {
    /// The name of the field.
    ///
    /// Can be missing, e.g. in case of an enum tuple struct variant.
    name: Option<F::String>,
    /// The kind of the field.
    ///
    /// This is either a direct layout bound
    /// or another recursive layout sub-struct.
    layout: Layout<F>,
}

impl FieldLayout {
    /// Creates a new field layout.
    pub fn new<N, L>(name: N, layout: L) -> Self
    where
        N: Into<Option<<MetaForm as Form>::String>>,
        L: Into<Layout>,
    {
        Self {
            name: name.into(),
            layout: layout.into(),
        }
    }
}

impl IntoCompact for FieldLayout {
    type Output = FieldLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        FieldLayout {
            name: self.name.map(|name| registry.register_string(name)),
            layout: self.layout.into_compact(registry),
        }
    }
}

/// The discriminant of an enum variant.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Discriminant(usize);

impl From<usize> for Discriminant {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// An enum storage layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct EnumLayout<F: Form = MetaForm> {
    /// The key where the discriminant is stored to dispatch the variants.
    dispatch_key: LayoutKey,
    /// The variants of the enum.
    variants: BTreeMap<Discriminant, StructLayout<F>>,
}

impl EnumLayout {
    /// Creates a new enum layout.
    pub fn new<K, V>(dispatch_key: K, variants: V) -> Self
    where
        K: Into<LayoutKey>,
        V: IntoIterator<Item = (Discriminant, StructLayout)>,
    {
        Self {
            dispatch_key: dispatch_key.into(),
            variants: variants.into_iter().collect(),
        }
    }
}

impl IntoCompact for EnumLayout {
    type Output = EnumLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        EnumLayout {
            dispatch_key: self.dispatch_key,
            variants: self
                .variants
                .into_iter()
                .map(|(discriminant, layout)| {
                    (discriminant, layout.into_compact(registry))
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_serialization_works() {
        let layout_key = LayoutKey::from(Key([0x01; 32]));
        let json = serde_json::to_string(&layout_key).unwrap();
        assert_eq!(
            json,
            "\"0x0101010101010101010101010101010101010101010101010101010101010101\"",
        );
    }
}
