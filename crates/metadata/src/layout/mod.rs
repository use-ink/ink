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

#[cfg(test)]
mod tests;

use crate::utils::serialize_as_byte_str;
use derive_more::From;
use ink_prelude::collections::btree_map::BTreeMap;
use ink_primitives::Key;
use scale_info::{
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    meta_type,
    IntoCompact,
    Registry,
    TypeInfo,
};

/// Represents the static storage layout of an ink! smart contract.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, From, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
#[serde(rename_all = "camelCase")]
pub enum Layout<F: Form = MetaForm> {
    /// An encoded cell.
    ///
    /// This is the only leaf node within the layout graph.
    /// All layout nodes have this node type as their leafs.
    ///
    /// This represents the encoding of a single cell mapped to a single key.
    Cell(CellLayout<F>),
    /// A layout that hashes values into the entire storage key space.
    ///
    /// This is commonly used by ink! hashmaps and similar data structures.
    Hash(HashLayout<F>),
    /// An array of associated storage cells encoded with a given type.
    ///
    /// This can also represent only a single cell.
    Array(ArrayLayout<F>),
    /// A struct layout with fields of different types.
    Struct(StructLayout<F>),
    /// An enum layout with a discriminant telling which variant is layed out.
    Enum(EnumLayout<F>),
}

/// A pointer into some storage region.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(transparent)]
pub struct LayoutKey {
    #[serde(serialize_with = "serialize_as_byte_str")]
    key: [u8; 32],
}

impl<'a> From<&'a Key> for LayoutKey {
    fn from(key: &'a Key) -> Self {
        Self {
            key: key.to_bytes(),
        }
    }
}

impl From<Key> for LayoutKey {
    fn from(key: Key) -> Self {
        Self {
            key: key.to_bytes(),
        }
    }
}

/// An encoded cell.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, From, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct CellLayout<F: Form = MetaForm> {
    /// The offset key into the storage.
    key: LayoutKey,
    /// The type of the encoded entity.
    ty: <F as Form>::TypeId,
}

impl CellLayout {
    /// Creates a new cell layout.
    pub fn new<T>(key: LayoutKey) -> Self
    where
        T: TypeInfo + 'static,
    {
        Self {
            key,
            ty: meta_type::<T>(),
        }
    }
}

impl IntoCompact for CellLayout {
    type Output = CellLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        CellLayout {
            key: self.key,
            ty: registry.register_type(&self.ty),
        }
    }
}

impl IntoCompact for Layout {
    type Output = Layout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        match self {
            Layout::Cell(encoded_cell) => {
                Layout::Cell(encoded_cell.into_compact(registry))
            }
            Layout::Hash(hash_layout) => Layout::Hash(hash_layout.into_compact(registry)),
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

/// A hashing layout potentially hitting all cells of the storage.
///
/// Every hashing layout has an offset and a strategy to compute its keys.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct HashLayout<F: Form = MetaForm> {
    /// The key offset used by the strategy.
    offset: LayoutKey,
    /// The hashing strategy to layout the underlying elements.
    strategy: HashingStrategy,
    /// The storage layout of the unbounded layout elements.
    layout: Box<Layout<F>>,
}

impl HashLayout {
    /// Creates a new unbounded layout.
    pub fn new<K, L>(offset: K, strategy: HashingStrategy, layout: L) -> Self
    where
        K: Into<LayoutKey>,
        L: Into<Layout>,
    {
        Self {
            offset: offset.into(),
            strategy,
            layout: Box::new(layout.into()),
        }
    }
}

impl IntoCompact for HashLayout {
    type Output = HashLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        HashLayout {
            offset: self.offset,
            strategy: self.strategy,
            layout: Box::new(self.layout.into_compact(registry)),
        }
    }
}

/// The unbounded hashing strategy.
///
/// The offset key is used as another postfix for the computation.
/// So the actual formula is: `hasher(prefix + encoded(key) + offset + postfix)`
/// Where `+` in this contexts means append of the byte slices.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct HashingStrategy {
    /// One of the supported crypto hashers.
    hasher: CryptoHasher,
    /// An optional prefix to the computed hash.
    #[serde(serialize_with = "serialize_as_byte_str")]
    prefix: Vec<u8>,
    /// An optional postfix to the computed hash.
    #[serde(serialize_with = "serialize_as_byte_str")]
    postfix: Vec<u8>,
}

impl HashingStrategy {
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
    /// The offset key of the array layout.
    ///
    /// This is the same key as the 0-th element of the array layout.
    offset: LayoutKey,
    /// The number of elements in the array layout.
    len: u32,
    /// The number of cells each element in the array layout consists of.
    cells_per_elem: u64,
    /// The layout of the elements stored in the array layout.
    layout: Box<Layout<F>>,
}

impl ArrayLayout {
    /// Creates an array layout with the given length.
    pub fn new<K, L>(at: K, len: u32, cells_per_elem: u64, layout: L) -> Self
    where
        K: Into<LayoutKey>,
        L: Into<Layout>,
    {
        Self {
            offset: at.into(),
            len,
            cells_per_elem,
            layout: Box::new(layout.into()),
        }
    }
}

impl IntoCompact for ArrayLayout {
    type Output = ArrayLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        ArrayLayout {
            offset: self.offset,
            len: self.len,
            cells_per_elem: self.cells_per_elem,
            layout: Box::new(self.layout.into_compact(registry)),
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
    name: Option<&'static str>,
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
        N: Into<Option<&'static str>>,
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
            name: self.name,
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
