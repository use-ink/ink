use codec::{Encode, Decode};

use frame_support::debug::{error, native};
use frame_support::traits::Randomness;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

/// contract extension for `FetchRandom`
pub struct FetchRandomExtension;

impl ChainExtension for FetchRandomExtension {
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
        where
            <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {

        match func_id {
            1101 => {
                let mut env = env.buf_in_buf_out();
                let random_seed: [u8; 32] = super::RandomnessCollectiveFlip::random_seed().0;
                let random_slice = random_seed.encode();
                native::trace!(
                    target: "runtime",
                    "[ChainExtension]|call|func_id:{:}",
                    func_id
                );
                env.write(&random_slice, false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;
            }

            _ => {
                error!("call an unregistered `func_id`, func_id:{:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        }
        Ok(RetVal::Converging(func_id))
    }

    fn enabled() -> bool {
        true
    }
}
