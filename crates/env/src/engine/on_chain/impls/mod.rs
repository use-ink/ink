
#[cfg(target_arch = "wasm32")]
mod pallet_contracts;
#[cfg(target_arch = "wasm32")]
pub use pallet_contracts::*;

#[cfg(target_arch = "riscv32")]
mod pallet_revive;
#[cfg(target_arch = "riscv32")]
pub use pallet_revive::*;
