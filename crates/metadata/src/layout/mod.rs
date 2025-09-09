// Copyright (C) Use Ink (UK) Ltd.
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
mod validate;

use core::fmt::Display;
pub use validate::ValidateLayout;

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
use scale::{
    Decode,
    Encode,
};
use scale_info::{
    IntoPortable,
    Registry,
    TypeInfo,
    form::{
        Form,
        MetaForm,
        PortableForm,
    },
    meta_type,
};
use schemars::JsonSchema;
use serde::{
    Deserialize,
    Serialize,
    de::{
        DeserializeOwned,
        Error,
    },
};

/// Represents the static storage layout of an ink! smart contract.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, From, Serialize, Deserialize, JsonSchema,
)]
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
    Leaf(LeafLayout<F>),
    /// The root cell defines the storage key for all sub-trees.
    Root(RootLayout<F>),
    /// A layout that hashes values into the entire storage key space.
    ///
    /// This is commonly used by ink! hashmaps and similar data structures.
    Hash(HashLayout<F>),
    /// An array of type associated with storage cell.
    Array(ArrayLayout<F>),
    /// A struct layout with fields of different types.
    Struct(StructLayout<F>),
    /// An enum layout with a discriminant telling which variant is laid out.
    Enum(EnumLayout<F>),
}

/// A pointer into some storage region.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, From, JsonSchema)]
pub struct LayoutKey {
    key: Key,
}

impl serde::Serialize for LayoutKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_hex::serialize(&self.key.encode(), serializer)
    }
}

impl<'de> serde::Deserialize<'de> for LayoutKey {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut arr = [0; 4];
        serde_hex::deserialize_check_len(d, serde_hex::ExpectedLen::Exact(&mut arr[..]))?;
        let key = Key::decode(&mut &arr[..])
            .map_err(|err| Error::custom(format!("Error decoding layout key: {err}")))?;
        Ok(key.into())
    }
}

impl<'a> From<&'a Key> for LayoutKey {
    fn from(key: &'a Key) -> Self {
        Self { key: *key }
    }
}

impl LayoutKey {
    /// Construct a custom layout key.
    pub fn new<T>(key: T) -> Self
    where
        T: Into<u32>,
    {
        Self { key: key.into() }
    }

    /// Returns the key of the layout key.
    pub fn key(&self) -> &Key {
        &self.key
    }
}

/// Sub-tree root.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, From, Serialize, Deserialize, JsonSchema,
)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct RootLayout<F: Form = MetaForm> {
    /// The root key of the sub-tree.
    #[schemars(with = "String")]
    root_key: LayoutKey,
    /// The storage layout of the unbounded layout elements.
    layout: Box<Layout<F>>,
    /// The type of the encoded entity.
    ty: <F as Form>::Type,
}

impl IntoPortable for RootLayout {
    type Output = RootLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        RootLayout {
            root_key: self.root_key,
            layout: Box::new(self.layout.into_portable(registry)),
            ty: registry.register_type(&self.ty),
        }
    }
}

impl RootLayout<MetaForm> {
    /// Creates a new root layout with empty root type.
    pub fn new_empty<L>(root_key: LayoutKey, layout: L) -> Self
    where
        L: Into<Layout<MetaForm>>,
    {
        Self::new::<L>(root_key, layout, meta_type::<()>())
    }
}

impl<F> RootLayout<F>
where
    F: Form,
{
    /// Create a new root layout
    pub fn new<L>(root_key: LayoutKey, layout: L, ty: <F as Form>::Type) -> Self
    where
        L: Into<Layout<F>>,
    {
        Self {
            root_key,
            layout: Box::new(layout.into()),
            ty,
        }
    }

    /// Returns the root key of the sub-tree.
    pub fn root_key(&self) -> &LayoutKey {
        &self.root_key
    }

    /// Returns the storage layout of the unbounded layout elements.
    pub fn layout(&self) -> &Layout<F> {
        &self.layout
    }

    /// Returns the type of the encoded entity.
    pub fn ty(&self) -> &F::Type {
        &self.ty
    }
}

/// A SCALE encoded cell.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, From, Serialize, Deserialize, JsonSchema,
)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct LeafLayout<F: Form = MetaForm> {
    /// The offset key into the storage.
    #[schemars(with = "String")]
    key: LayoutKey,
    /// The type of the encoded entity.
    ty: <F as Form>::Type,
}

impl LeafLayout {
    /// Creates a new cell layout.
    pub fn from_key<T>(key: LayoutKey) -> Self
    where
        T: TypeInfo + 'static,
    {
        Self {
            key,
            ty: meta_type::<T>(),
        }
    }
}

impl IntoPortable for LeafLayout {
    type Output = LeafLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        LeafLayout {
            key: self.key,
            ty: registry.register_type(&self.ty),
        }
    }
}

impl IntoPortable for Layout {
    type Output = Layout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        match self {
            Layout::Leaf(encoded_cell) => {
                Layout::Leaf(encoded_cell.into_portable(registry))
            }
            Layout::Root(encoded_cell) => {
                Layout::Root(encoded_cell.into_portable(registry))
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

impl<F> LeafLayout<F>
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

    pub fn new(key: LayoutKey, ty: <F as Form>::Type) -> Self {
        Self { key, ty }
    }
}

/// A hashing layout potentially hitting all cells of the storage.
///
/// Every hashing layout has an offset and a strategy to compute its keys.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct HashLayout<F: Form = MetaForm> {
    /// The key offset used by the strategy.
    #[schemars(with = "String")]
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum CryptoHasher {
    /// The BLAKE-2 crypto hasher with an output of 256 bits.
    Blake2x256,
    /// The SHA-2 crypto hasher with an output of 256 bits.
    Sha2x256,
    /// The KECCAK crypto hasher with an output of 256 bits.
    Keccak256,
}

/// A layout for an array of associated cells with the same encoding.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
pub struct ArrayLayout<F: Form = MetaForm> {
    /// The offset key of the array layout.
    ///
    /// This is the same key as the element at index 0 of the array layout.
    #[schemars(with = "String")]
    offset: LayoutKey,
    /// The number of elements in the array layout.
    len: u32,
    /// The layout of the elements stored in the array layout.
    layout: Box<Layout<F>>,
}

impl ArrayLayout {
    /// Creates an array layout with the given length.
    pub fn new<K, L>(at: K, len: u32, layout: L) -> Self
    where
        K: Into<LayoutKey>,
        L: Into<Layout>,
    {
        Self {
            offset: at.into(),
            len,
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
    /// This is the same key as the element at index 0 of the array layout.
    pub fn offset(&self) -> &LayoutKey {
        &self.offset
    }

    /// Returns the number of elements in the array layout.
    pub fn len(&self) -> u32 {
        self.len
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
            layout: Box::new(self.layout.into_portable(registry)),
        }
    }
}

/// A struct layout with consecutive fields of different layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct StructLayout<F: Form = MetaForm> {
    /// The name of the struct.
    name: F::String,
    /// The fields of the struct layout.
    fields: Vec<FieldLayout<F>>,
}

impl<F> StructLayout<F>
where
    F: Form,
{
    /// Creates a new struct layout.
    pub fn new<N, T>(name: N, fields: T) -> Self
    where
        N: Into<F::String>,
        T: IntoIterator<Item = FieldLayout<F>>,
    {
        Self {
            name: name.into(),
            fields: fields.into_iter().collect(),
        }
    }

    /// Returns the name of the struct.
    pub fn name(&self) -> &F::String {
        &self.name
    }
    /// Returns the fields of the struct layout.
    pub fn fields(&self) -> &[FieldLayout<F>] {
        &self.fields
    }
}

impl IntoPortable for StructLayout {
    type Output = StructLayout<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        StructLayout {
            name: self.name.to_string(),
            fields: self
                .fields
                .into_iter()
                .map(|field| field.into_portable(registry))
                .collect::<Vec<_>>(),
        }
    }
}

/// The layout for a particular field of a struct layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct FieldLayout<F: Form = MetaForm> {
    /// The name of the field.
    name: F::String,
    /// The kind of the field.
    ///
    /// This is either a direct layout bound
    /// or another recursive layout sub-struct.
    layout: Layout<F>,
}

impl<F> FieldLayout<F>
where
    F: Form,
{
    /// Creates a new custom field layout.
    pub fn new<N, L>(name: N, layout: L) -> Self
    where
        N: Into<F::String>,
        L: Into<Layout<F>>,
    {
        Self {
            name: name.into(),
            layout: layout.into(),
        }
    }

    /// Returns the name of the field.
    pub fn name(&self) -> &F::String {
        &self.name
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
            name: self.name.to_string(),
            layout: self.layout.into_portable(registry),
        }
    }
}

/// The discriminant of an enum variant.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
pub struct EnumLayout<F: Form = MetaForm> {
    /// The name of the Enum.
    name: F::String,
    /// The key where the discriminant is stored to dispatch the variants.
    #[schemars(with = "String")]
    dispatch_key: LayoutKey,
    /// The variants of the enum.
    variants: BTreeMap<Discriminant, StructLayout<F>>,
}

impl EnumLayout {
    /// Creates a new enum layout.
    pub fn new<N, K, V>(name: N, dispatch_key: K, variants: V) -> Self
    where
        N: Into<<MetaForm as Form>::String>,
        K: Into<LayoutKey>,
        V: IntoIterator<Item = (Discriminant, StructLayout)>,
    {
        Self {
            name: name.into(),
            dispatch_key: dispatch_key.into(),
            variants: variants.into_iter().collect(),
        }
    }
}

impl<F> EnumLayout<F>
where
    F: Form,
{
    /// Returns the name of the field.
    pub fn name(&self) -> &F::String {
        &self.name
    }

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
            name: self.name.to_string(),
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

/// An error that can occur during ink! metadata generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    /// Storage keys of two types intersect
    Collision(String, String),
}

impl Display for MetadataError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Collision(prev_path, curr_path) => {
                write!(
                    f,
                    "storage key collision occurred for `{}`. \
                    The same storage key is occupied by the `{}`.",
                    curr_path,
                    if prev_path.is_empty() {
                        "contract storage"
                    } else {
                        prev_path
                    }
                )
            }
        }
    }
}

#[test]
fn valid_error_message() {
    assert_eq!(
        MetadataError::Collision("".to_string(), "Contract.c:".to_string()).to_string(),
        "storage key collision occurred for `Contract.c:`. \
        The same storage key is occupied by the `contract storage`."
    );
    assert_eq!(
        MetadataError::Collision("Contract.a:".to_string(), "Contract.c:".to_string())
            .to_string(),
        "storage key collision occurred for `Contract.c:`. \
        The same storage key is occupied by the `Contract.a:`."
    )
}
