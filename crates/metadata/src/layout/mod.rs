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

#[cfg(test)]
mod tests;

use crate::{
    serde_hex,
    utils::{
        deserialize_from_byte_str,
        serialize_as_byte_str,
    },
};
use derive_more::From;
use ink_prelude::collections::btree_map::BTreeMap;
use ink_primitives::Key;
use scale_info::{
    form::{
        Form,
        MetaForm,
        PortableForm,
    },
    meta_type,
    IntoPortable,
    Registry,
    TypeInfo,
};
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

/// Represents the static storage layout of an ink! smart contract.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, From, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct LayoutKey {
    key: [u8; 32],
}

impl serde::Serialize for LayoutKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_hex::serialize(&self.key, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for LayoutKey {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut arr = [0; 32];
        serde_hex::deserialize_check_len(d, serde_hex::ExpectedLen::Exact(&mut arr[..]))?;
        Ok(arr.into())
    }
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

impl LayoutKey {
    /// Returns the underlying bytes of the layout key.
    pub fn to_bytes(&self) -> &[u8] {
        &self.key
    }
}

/// A SCALE encoded cell.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, From, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct CellLayout<F: Form = MetaForm> {
    /// The offset key into the storage.
    key: LayoutKey,
    /// The type of the encoded entity.
    ty: <F as Form>::Type,
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

impl IntoPortable for CellLayout {
    type Output = CellLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        CellLayout {
            key: self.key,
            ty: registry.register_type(&self.ty),
        }
    }
}

impl IntoPortable for Layout {
    type Output = Layout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        match self {
            Layout::Cell(encoded_cell) => {
                Layout::Cell(encoded_cell.into_portable(registry))
            }
            Layout::Hash(hash_layout) => {
                Layout::Hash(hash_layout.into_portable(registry))
            }
            Layout::Array(array_layout) => {
                Layout::Array(array_layout.into_portable(registry))
            }
            Layout::Struct(struct_layout) => {
                Layout::Struct(struct_layout.into_portable(registry))
            }
            Layout::Enum(enum_layout) => {
                Layout::Enum(enum_layout.into_portable(registry))
            }
        }
    }
}

impl<F> CellLayout<F>
where
    F: Form,
{
    /// Returns the offset key into the storage.
    pub fn key(&self) -> &LayoutKey {
        &self.key
    }

    /// Returns the type of the encoded entity.
    pub fn ty(&self) -> &F::Type {
        &self.ty
    }
}

/// A hashing layout potentially hitting all cells of the storage.
///
/// Every hashing layout has an offset and a strategy to compute its keys.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct HashLayout<F: Form = MetaForm> {
    /// The key offset used by the strategy.
    offset: LayoutKey,
    /// The hashing strategy to layout the underlying elements.
    strategy: HashingStrategy,
    /// The storage layout of the unbounded layout elements.
    layout: Box<Layout<F>>,
}

impl IntoPortable for HashLayout {
    type Output = HashLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        HashLayout {
            offset: self.offset,
            strategy: self.strategy,
            layout: Box::new(self.layout.into_portable(registry)),
        }
    }
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

impl<F> HashLayout<F>
where
    F: Form,
{
    /// Returns the key offset used by the strategy.
    pub fn offset(&self) -> &LayoutKey {
        &self.offset
    }

    /// Returns the hashing strategy to layout the underlying elements.
    pub fn strategy(&self) -> &HashingStrategy {
        &self.strategy
    }

    /// Returns the storage layout of the unbounded layout elements.
    pub fn layout(&self) -> &Layout<F> {
        &self.layout
    }
}

/// The unbounded hashing strategy.
///
/// The offset key is used as another postfix for the computation.
/// So the actual formula is: `hasher(prefix + encoded(key) + offset + postfix)`
/// Where `+` in this contexts means append of the byte slices.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HashingStrategy {
    /// One of the supported crypto hashers.
    hasher: CryptoHasher,
    /// An optional prefix to the computed hash.
    #[serde(
        serialize_with = "serialize_as_byte_str",
        deserialize_with = "deserialize_from_byte_str"
    )]
    prefix: Vec<u8>,
    /// An optional postfix to the computed hash.
    #[serde(
        serialize_with = "serialize_as_byte_str",
        deserialize_with = "deserialize_from_byte_str"
    )]
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

    /// Returns the supported crypto hasher.
    pub fn hasher(&self) -> &CryptoHasher {
        &self.hasher
    }

    /// Returns the optional prefix to the computed hash.
    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    /// Returns the optional postfix to the computed hash.
    pub fn postfix(&self) -> &[u8] {
        &self.postfix
    }
}

/// One of the supported crypto hashers.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CryptoHasher {
    /// The BLAKE-2 crypto hasher with an output of 256 bits.
    Blake2x256,
    /// The SHA-2 crypto hasher with an output of 256 bits.
    Sha2x256,
    /// The KECCAK crypto hasher with an output of 256 bits.
    Keccak256,
}

/// A layout for an array of associated cells with the same encoding.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
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

#[allow(clippy::len_without_is_empty)]
impl<F> ArrayLayout<F>
where
    F: Form,
{
    /// Returns the offset key of the array layout.
    ///
    /// This is the same key as the 0-th element of the array layout.
    pub fn offset(&self) -> &LayoutKey {
        &self.offset
    }

    /// Returns the number of elements in the array layout.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns he number of cells each element in the array layout consists of.
    pub fn cells_per_elem(&self) -> u64 {
        self.cells_per_elem
    }

    /// Returns the layout of the elements stored in the array layout.
    pub fn layout(&self) -> &Layout<F> {
        &self.layout
    }
}

impl IntoPortable for ArrayLayout {
    type Output = ArrayLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        ArrayLayout {
            offset: self.offset,
            len: self.len,
            cells_per_elem: self.cells_per_elem,
            layout: Box::new(self.layout.into_portable(registry)),
        }
    }
}

/// A struct layout with consecutive fields of different layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
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

impl<F> StructLayout<F>
where
    F: Form,
{
    /// Returns the fields of the struct layout.
    pub fn fields(&self) -> &[FieldLayout<F>] {
        &self.fields
    }
}

impl IntoPortable for StructLayout {
    type Output = StructLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        StructLayout {
            fields: self
                .fields
                .into_iter()
                .map(|field| field.into_portable(registry))
                .collect::<Vec<_>>(),
        }
    }
}

/// The layout for a particular field of a struct layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
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
        N: Into<Option<&'static str>>,
        L: Into<Layout>,
    {
        Self {
            name: name.into(),
            layout: layout.into(),
        }
    }
}

impl<F> FieldLayout<F>
where
    F: Form,
{
    /// Returns the name of the field.
    ///
    /// Can be missing, e.g. in case of an enum tuple struct variant.
    pub fn name(&self) -> Option<&F::String> {
        self.name.as_ref()
    }

    /// Returns the kind of the field.
    ///
    /// This is either a direct layout bound
    /// or another recursive layout sub-struct.
    pub fn layout(&self) -> &Layout<F> {
        &self.layout
    }
}

impl IntoPortable for FieldLayout {
    type Output = FieldLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        FieldLayout {
            name: self.name.map(|name| name.into_portable(registry)),
            layout: self.layout.into_portable(registry),
        }
    }
}

/// The discriminant of an enum variant.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Discriminant(usize);

impl From<usize> for Discriminant {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Discriminant {
    /// Returns the value of the discriminant
    pub fn value(&self) -> usize {
        self.0
    }
}

/// An enum storage layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
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

impl<F> EnumLayout<F>
where
    F: Form,
{
    /// Returns the key where the discriminant is stored to dispatch the variants.
    pub fn dispatch_key(&self) -> &LayoutKey {
        &self.dispatch_key
    }

    /// Returns the variants of the enum.
    pub fn variants(&self) -> &BTreeMap<Discriminant, StructLayout<F>> {
        &self.variants
    }
}

impl IntoPortable for EnumLayout {
    type Output = EnumLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        EnumLayout {
            dispatch_key: self.dispatch_key,
            variants: self
                .variants
                .into_iter()
                .map(|(discriminant, layout)| {
                    (discriminant, layout.into_portable(registry))
                })
                .collect(),
        }
    }
}
