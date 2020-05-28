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

/// Serializes the given bytes as byte string.
fn serialize_as_byte_str<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bytes = bytes.as_ref();
    if bytes.is_empty() {
        // Return empty string without prepended `0x`.
        return serializer.serialize_str("")
    }
    let mut hex = String::with_capacity(bytes.len() * 2 + 2);
    write!(hex, "0x").expect("failed writing to string");
    for byte in bytes {
        write!(hex, "{:02x}", byte).expect("failed writing to string");
    }
    serializer.serialize_str(&hex)
}

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
    /// An encoded cell.
    ///
    /// This is the only leaf node within the layout graph.
    /// All layout nodes have this node type as their leafs.
    ///
    /// This represents the encoding of a single cell mapped to a single key.
    Cell(CellLayout<F>),
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

/// A pointer into some storage region.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(transparent)]
pub struct LayoutKey {
    #[serde(serialize_with = "serialize_as_byte_str")]
    key: [u8; 32],
}

impl From<Key> for LayoutKey {
    fn from(key: Key) -> Self {
        let mut arr = [0x00; 32];
        arr.copy_from_slice(key.as_bytes());
        Self { key: arr }
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
    pub fn new<T, K>(key: K) -> Self
    where
        T: Metadata,
        K: Into<LayoutKey>,
    {
        Self {
            key: key.into(),
            ty: <T as Metadata>::meta_type(),
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
    /// The key offset used by the strategy.
    offset: LayoutKey,
    /// The actual strategy to compute the unbounded keys.
    strategy: UnboundedStrategy,
    /// The storage layout of the unbounded layout elements.
    layout: Box<Layout<F>>,
}

impl UnboundedLayout {
    /// Creates a new unbounded layout.
    pub fn new<K, S, L>(offset: K, strategy: S, layout: L) -> Self
    where
        K: Into<LayoutKey>,
        S: Into<UnboundedStrategy>,
        L: Into<Layout>,
    {
        Self {
            offset: offset.into(),
            strategy: strategy.into(),
            layout: Box::new(layout.into()),
        }
    }
}

impl IntoCompact for UnboundedLayout {
    type Output = UnboundedLayout<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        UnboundedLayout {
            offset: self.offset,
            strategy: self.strategy,
            layout: Box::new(self.layout.into_compact(registry)),
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
    #[serde(serialize_with = "serialize_as_byte_str")]
    prefix: Vec<u8>,
    /// An optional postfix to the computed hash.
    #[serde(serialize_with = "serialize_as_byte_str")]
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
    pub fn new<T, K, L>(at: K, len: u32, cells_per_elem: u64, layout: L) -> Self
    where
        T: Metadata,
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

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    pub struct NamedFieldsStruct {
        a: i32,
        b: i64,
    }

    impl StorageLayout for NamedFieldsStruct {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            StructLayout::new(vec![
                FieldLayout::new(
                    "a",
                    CellLayout::new::<i32, _>(LayoutKey::from(key_ptr.advance_by(1))),
                ),
                FieldLayout::new(
                    "b",
                    CellLayout::new::<i64, _>(LayoutKey::from(key_ptr.advance_by(1))),
                ),
            ])
            .into()
        }
    }

    #[test]
    fn named_fields_work() {
        let layout = <NamedFieldsStruct as StorageLayout>::layout(&mut KeyPtr::from(
            Key([0x00; 32]),
        ));
        let mut registry = Registry::new();
        let compacted = layout.into_compact(&mut registry);
        let json = serde_json::to_value(&compacted).unwrap();
        let expected = serde_json::json! {
            {
                "Struct": {
                    "fields": [
                        {
                            "layout": {
                                "Cell": {
                                    "key": "0x\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000000",
                                    "ty": 1,
                                }
                            },
                            "name": 1,
                        },
                        {
                            "layout": {
                                "Cell": {
                                    "key": "0x\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000001",
                                    "ty": 2,
                                }
                            },
                            "name": 2,
                        }
                    ]
                }
            }
        };
        assert_eq!(json, expected);
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    pub struct TupleStruct(i32, i64);

    impl StorageLayout for TupleStruct {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            StructLayout::new(vec![
                FieldLayout::new(
                    None,
                    CellLayout::new::<i32, _>(LayoutKey::from(key_ptr.advance_by(1))),
                ),
                FieldLayout::new(
                    None,
                    CellLayout::new::<i64, _>(LayoutKey::from(key_ptr.advance_by(1))),
                ),
            ])
            .into()
        }
    }

    #[test]
    fn tuple_struct_work() {
        let layout =
            <TupleStruct as StorageLayout>::layout(&mut KeyPtr::from(Key([0x00; 32])));
        let mut registry = Registry::new();
        let compacted = layout.into_compact(&mut registry);
        let json = serde_json::to_value(&compacted).unwrap();
        let expected = serde_json::json! {
            {
                "Struct": {
                    "fields": [
                        {
                            "layout": {
                                "Cell": {
                                    "key": "0x\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000000",
                                    "ty": 1,
                                }
                            },
                            "name": null,
                        },
                        {
                            "layout": {
                                "Cell": {
                                    "key": "0x\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000000\
                                        0000000000000001",
                                    "ty": 2,
                                }
                            },
                            "name": null,
                        }
                    ]
                }
            }
        };
        assert_eq!(json, expected);
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum ClikeEnum {
        A,
        B,
        C,
    }

    impl StorageLayout for ClikeEnum {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            EnumLayout::new(
                key_ptr.advance_by(1),
                vec![
                    (Discriminant(0), StructLayout::new(vec![])),
                    (Discriminant(1), StructLayout::new(vec![])),
                    (Discriminant(2), StructLayout::new(vec![])),
                ],
            )
            .into()
        }
    }

    #[test]
    fn clike_enum_work() {
        let layout =
            <ClikeEnum as StorageLayout>::layout(&mut KeyPtr::from(Key([0x00; 32])));
        let mut registry = Registry::new();
        let compacted = layout.into_compact(&mut registry);
        let json = serde_json::to_value(&compacted).unwrap();
        let expected = serde_json::json! {
            {
                "Enum": {
                    "dispatch_key": "0x\
                        0000000000000000\
                        0000000000000000\
                        0000000000000000\
                        0000000000000000",
                    "variants": {
                        "0": {
                            "fields": [],
                        },
                        "1": {
                            "fields": [],
                        },
                        "2": {
                            "fields": [],
                        },
                    }
                }
            }
        };
        assert_eq!(json, expected);
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum MixedEnum {
        ClikeVariant,
        TupleVariant(i32, i64),
        StructVariant { a: i32, b: i64 },
    }

    impl StorageLayout for MixedEnum {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            EnumLayout::new(
                key_ptr.advance_by(1),
                vec![
                    (Discriminant(0), StructLayout::new(vec![])),
                    {
                        let mut variant_key_ptr = KeyPtr::from(key_ptr.advance_by(0));
                        (
                            Discriminant(1),
                            StructLayout::new(vec![
                                FieldLayout::new(
                                    None,
                                    CellLayout::new::<i32, _>(LayoutKey::from(
                                        variant_key_ptr.advance_by(1),
                                    )),
                                ),
                                FieldLayout::new(
                                    None,
                                    CellLayout::new::<i64, _>(LayoutKey::from(
                                        variant_key_ptr.advance_by(1),
                                    )),
                                ),
                            ]),
                        )
                    },
                    {
                        let mut variant_key_ptr = KeyPtr::from(key_ptr.advance_by(0));
                        (
                            Discriminant(2),
                            StructLayout::new(vec![
                                FieldLayout::new(
                                    "a",
                                    CellLayout::new::<i32, _>(LayoutKey::from(
                                        variant_key_ptr.advance_by(1),
                                    )),
                                ),
                                FieldLayout::new(
                                    "b",
                                    CellLayout::new::<i64, _>(LayoutKey::from(
                                        variant_key_ptr.advance_by(1),
                                    )),
                                ),
                            ]),
                        )
                    },
                ],
            )
            .into()
        }
    }

    #[test]
    fn mixed_enum_work() {
        let layout =
            <MixedEnum as StorageLayout>::layout(&mut KeyPtr::from(Key([0x00; 32])));
        let mut registry = Registry::new();
        let compacted = layout.into_compact(&mut registry);
        let json = serde_json::to_value(&compacted).unwrap();
        let expected = serde_json::json! {
            {
                "Enum": {
                    "dispatch_key": "0x\
                        0000000000000000\
                        0000000000000000\
                        0000000000000000\
                        0000000000000000",
                    "variants": {
                        "0": {
                            "fields": [],
                        },
                        "1": {
                            "fields": [
                                {
                                    "layout": {
                                        "Cell": {
                                            "key": "0x\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000001",
                                            "ty": 1,
                                        }
                                    },
                                    "name": null,
                                },
                                {
                                    "layout": {
                                        "Cell": {
                                            "key": "0x\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000002",
                                            "ty": 2,
                                        }
                                    },
                                    "name": null,
                                }
                            ],
                        },
                        "2": {
                            "fields": [
                                {
                                    "layout": {
                                        "Cell": {
                                            "key": "0x\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000001",
                                            "ty": 1,
                                        }
                                    },
                                    "name": 1,
                                },
                                {
                                    "layout": {
                                        "Cell": {
                                            "key": "0x\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000000\
                                                0000000000000002",
                                            "ty": 2,
                                        }
                                    },
                                    "name": 2,
                                }
                            ],
                        },
                    }
                }
            }
        };
        assert_eq!(json, expected);
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Blake2x256Hasher;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    pub struct UnboundedMapping {
        hasher: Blake2x256Hasher,
        kvs: Vec<(i32, bool)>,
    }

    impl StorageLayout for UnboundedMapping {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            let root_key = key_ptr.advance_by(1);
            UnboundedLayout::new(
                root_key,
                UnboundedStrategy::Hashing(UnboundedHashingStrategy::new(
                    CryptoHasher::Blake2x256,
                    b"ink storage hashmap".to_vec(),
                    Vec::new(),
                )),
                CellLayout::new::<(i32, bool), _>(root_key),
            )
            .into()
        }
    }

    #[test]
    fn unbounded_layout_works() {
        let layout = <UnboundedMapping as StorageLayout>::layout(&mut KeyPtr::from(Key(
            [0x00; 32],
        )));
        let mut registry = Registry::new();
        let compacted = layout.into_compact(&mut registry);
        let json = serde_json::to_value(&compacted).unwrap();
        let expected = serde_json::json! {
            {
                "Unbounded": {
                    "layout": {
                        "Cell": {
                            "key": "0x\
                                0000000000000000\
                                0000000000000000\
                                0000000000000000\
                                0000000000000000",
                            "ty": 1
                        }
                    },
                    "offset": "0x\
                        0000000000000000\
                        0000000000000000\
                        0000000000000000\
                        0000000000000000",
                    "strategy": {
                        "Hashing": {
                            "hasher": "Blake2x256",
                            "prefix": "0x696e6b2073746f7261676520686173686d6170",
                            "postfix": "",
                        }
                    }
                }
            }
        };
        assert_eq!(json, expected);
    }
}
