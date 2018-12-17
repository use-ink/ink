#[cfg(all(test, feature = "test-env"))]
mod tests;

mod hash_map;

pub use self::hash_map::{
	HashMap,
};
