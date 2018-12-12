//! Hashing utilities around the Keccak256 hasher.

use tiny_keccak;

/// Keccak256 hasher.
pub struct Keccak256Hasher {
	/// The internal keccak hasher.
	hasher: tiny_keccak::Keccak,
}

impl Default for Keccak256Hasher {
	fn default() -> Self {
		Keccak256Hasher{
			hasher: tiny_keccak::Keccak::new_keccak256()
		}
	}
}

impl Keccak256Hasher {
	/// Returns the hash value for the values written so far.
	///
	/// If you need to start a fresh hash value, you will have to create a new hasher.
	pub fn finish(self) -> [u8; 32] {
		let mut res = [0; 32];
		self.hasher.finalize(&mut res);
		res
	}

	/// Writes some data into the hasher.
	pub fn write(&mut self, bytes: &[u8]) {
		self.hasher.update(bytes)
	}
}

/// Types implementing this trait are keccak256 hashable.
pub trait HashAsKeccak256 {
	/// Hash `self` with the given hasher.
	fn hash_as_keccak256(&self, hasher: &mut Keccak256Hasher);
}

impl<T> HashAsKeccak256 for T
where
	T: ?Sized + AsRef<[u8]>
{
	fn hash_as_keccak256(&self, hasher: &mut Keccak256Hasher) {
		hasher.write(self.as_ref())
	}
}

/// Returns the keccak-256 hash for the given byte slice.
pub fn keccak256<T>(val: &T) -> [u8; 32]
where
	T: ?Sized + HashAsKeccak256
{
	let mut hasher = Keccak256Hasher::default();
	val.hash_as_keccak256(&mut hasher);
	hasher.finish()
}

/// Copies the keccak-256 hash into the given destination byte slice.
pub fn keccak256_inplace(source: &[u8], dest: &mut [u8; 32]) {
	dest.copy_from_slice(&keccak256(source));
}
