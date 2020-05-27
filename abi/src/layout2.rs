use ink_prelude::collections::btree_map::BTreeMap;
use ink_primitives::Key;
use type_metadata::{
    form::{
        Form,
        MetaForm,
    },
    Metadata,
};

pub struct KeyPtr(::ink_primitives::Key);

/// Implemented by types that have a storage layout.
pub trait StorageLayout {
    fn layout(key_ptr: &mut KeyPtr) -> Layout;
}

/// Represents the static storage layout of an ink! smart contract.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
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
            hasher: hasher.into(),
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct LayoutKey {
    key: [u8; 32],
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
    pub fn array<T, K>(at: K, len: u32) -> Self
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
    /// The variants of the enum.
    variants: BTreeMap<Discriminant, VariantLayout<F>>,
}

impl EnumLayout {
    /// Creates a new enum layout.
    pub fn new<V>(variants: V) -> Self
    where
        V: IntoIterator<Item = (Discriminant, VariantLayout)>,
    {
        Self {
            variants: variants.into_iter().collect(),
        }
    }
}

/// A variant storage layout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(bound = "F::TypeId: serde::Serialize")]
pub struct VariantLayout<F: Form = MetaForm> {
    /// The discriminant for the variant.
    discriminant: Discriminant,
    /// The fields of the discriminant.
    ///
    /// This can be empty for unit variants.
    /// Field layouts has an optional name to represent tuple struct variants.
    fields: Vec<FieldLayout<F>>,
}

impl VariantLayout {
    /// Creates a new variant layout.
    pub fn new<I>(discriminant: Discriminant, fields: I) -> Self
    where
        I: IntoIterator<Item = FieldLayout>,
    {
        Self {
            discriminant,
            fields: fields.into_iter().collect(),
        }
    }
}
