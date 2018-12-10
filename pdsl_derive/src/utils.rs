use crate::error::Result;

/// Returns `true` if the given set of attributes contain the
/// expected attribute that is represented by the given string
/// segments separated by `::`.
/// 
/// # Input
/// 
/// - `expected = "Hello"` returns `true` if there is an
///   attribute `#[Hello]`.
/// - `expected = "foo::bar"` returns `true` if there is an
///   attribute `#[foo::bar]`.
pub fn has_attribute(attrs: &[syn::Attribute], expected: &str) -> bool {
	let expected = expected.split("::").collect::<Vec<&str>>();
	attrs.iter().any(|attr| {
		if attr.path.segments.len() != 2 { return false }
		attr
			.path
			.segments
			.iter()
			.zip(expected.iter())
			.all(|(act, exp)| act.ident == exp)
	})
}

/// 256-bit byte blob.
pub type Bytes256 = [u8; 32];

mod keccak {
	use super::Bytes256;

	/// 256-bit hash result of the Keccak hasher.
	pub struct Hash(Bytes256);

	/// Returns the keccak hash of the given byte slice.
	pub fn hash(data: &[u8]) -> Hash {
		Hash([0x1; 32]) // TODO: depend on tiny_keccak
	}
}

pub struct KeccakHasher {
	buffer: Vec<u8>,
	temp: String,
}

impl KeccakHasher {
	pub fn new() -> Self {
		KeccakHasher{ buffer: Vec::new(), temp: String::new() }
	}

	pub fn push(&mut self, data: &[u8]) {
		self.buffer.extend(data)
	}

	pub fn push_const<T>(&mut self, data: T) -> std::fmt::Result
	where
		T: std::fmt::Display
	{
		use std::fmt::Write;
		self.temp.clear();
		write!(self.temp, "{}", data)?;
		Ok(
			self.buffer.extend(self.temp.as_bytes())
		)
	}

	pub fn finalize(self) -> keccak::Hash {
		keccak::hash(&self.buffer)
	}
}

/// Types that implement this trait can be used to form
/// function signatures for public contract methods.
///
/// These hashes are used to dispatch a public contract method
/// that has been called from outside.
pub trait SignatureHash {
	/// Applies the keccak hash of `self` for the given keccak hasher.
	fn keccak_hash(hasher: &mut KeccakHasher);
}

/// The signature of a contract function contains its
/// name and its parameters.
pub struct FuncSig {
	/// The name of the function.
	name: syn::Ident,
	/// The set of function arguments.
	inputs: Vec<syn::FnArg>
}

/// A function identifier that is used to dispatch a called
/// contract function in the `call` extern function.
pub struct FuncId(Bytes256);

impl FuncId {
	/// Create a new function identifier for the given contracta
	/// and function signature.
	pub fn new(contract_id: syn::Ident, fn_sig: FuncSig) -> Result<Self> {
		// use keccak hash of `bytes(contract) (+) bytes(fn_sig)`
		unimplemented!()
	}
}

/// A function invocation is the pair of a function identifier
/// and associated encoded input data given as raw byte slice.
pub struct FuncInvocation<'a> {
	id: FuncId,
	args: &'a [u8]
}

/// Storage key used to store and load on-chain data.
pub struct StorageKey(Bytes256);

impl StorageKey {
	/// Create a new storage key for the given contract and field.
	pub fn new(contract: syn::Ident, field_name: syn::Ident) -> Result<Self> {
		// use keccak hash of `bytes(contract) (+) bytes(field_name)`
		unimplemented!()
	}
}
