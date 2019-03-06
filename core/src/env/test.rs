//! Public api to interact with the special testing environment.

use super::ContractEnv;

/// Returns the total number of reads to all storage entries.
pub fn total_reads() -> u64 {
	ContractEnv::total_reads()
}

/// Returns the total number of writes to all storage entries.
pub fn total_writes() -> u64 {
	ContractEnv::total_writes()
}
