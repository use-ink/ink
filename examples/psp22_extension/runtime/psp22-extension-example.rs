use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	log::{error, trace},
	pallet_prelude::*,
	traits::fungibles::{
		approvals::{Inspect as AllowanceInspect, Mutate as AllowanceMutate},
		Inspect, InspectMetadata, Transfer,
	},
};
use pallet_assets::{self, WeightInfo};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22BalanceOfInput<AssetId, AccountId> {
	pub asset_id: AssetId,
	pub owner: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22AllowanceInput<AssetId, AccountId> {
	pub asset_id: AssetId,
	pub owner: AccountId,
	pub spender: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22TransferInput<AssetId, AccountId, Balance> {
	pub asset_id: AssetId,
	pub to: AccountId,
	pub value: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22TransferFromInput<AssetId, AccountId, Balance> {
	pub asset_id: AssetId,
	pub from: AccountId,
	pub to: AccountId,
	pub value: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
pub struct Psp22ApproveInput<AssetId, AccountId, Balance> {
	pub asset_id: AssetId,
	pub spender: AccountId,
	pub value: Balance,
}

pub struct Psp22Extension;

impl<T> ChainExtension<T> for Psp22Extension
where
	T: SysConfig + pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		match func_id {
			// Note: We use the PSP22 interface selectors as function IDs,
			// there is no need but it makes sense from a convention perspective.

			// PSP22 Metadata interfaces
			0x3d261bd4 | 0x34205be5 | 0x7271b782 => {
				let mut env = env.buf_in_buf_out();
				let asset_id = env.read_as()?;
				let result = match func_id {
					// PSP22Metadata::token_name
					0x3d261bd4 =>
						<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::name(&asset_id)
							.encode(),
					// PSP22Metadata::token_symbol
					0x34205be5 =>
						<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::symbol(
							&asset_id,
						)
						.encode(),
					// PSP22Metadata::token_decimals
					0x7271b782 =>
						<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::decimals(
							&asset_id,
						)
						.encode(),
					_ => unreachable!(),
				};
				trace!(
					target: "runtime",
					"[ChainExtension] PSP22Metadata::{:?}",
					func_id
				);
				env.write(&result, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call PSP22Metadata")
				})?;
			},

			// PSP22 interface queries
			0x162df8c2 | 0x6568382f | 0x4d47d921 => {
				let mut env = env.buf_in_buf_out();
				let result = match func_id {
					// PSP22::total_supply
					0x162df8c2 => {
						let asset_id = env.read_as()?;
						<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::total_issuance(
							asset_id,
						)
						.encode()
					},
					// PSP22::balance_of
					0x6568382f => {
						let input: Psp22BalanceOfInput<T::AssetId, T::AccountId> = env.read_as()?;
						<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(
							input.asset_id,
							&input.owner,
						)
						.encode()
					},
					// PSP22::allowance
					0x4d47d921 => {
						let input: Psp22AllowanceInput<T::AssetId, T::AccountId> = env.read_as()?;
						<pallet_assets::Pallet<T> as AllowanceInspect<T::AccountId>>::allowance(
							input.asset_id,
							&input.owner,
							&input.spender,
						)
						.encode()
					},
					_ => unreachable!(),
				};
				trace!(
					target: "runtime",
					"[ChainExtension] PSP22::{:?}",
					func_id
				);
				env.write(&result, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call PSP22 query")
				})?;
			},

			// P2P22:transfer
			0xdb20f9f5 => {
				let mut env = env.buf_in_buf_out();

				let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
				let overhead =
					<T as pallet_contracts::Config>::Schedule::get().host_fn_weights.debug_message;
				let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
				trace!(
					target: "runtime",
					"[ChainExtension]|call|transfer / charge_weight:{:?}",
					charged_weight
				);

				let input: Psp22TransferInput<T::AssetId, T::AccountId, T::Balance> =
					env.read_as()?;
				let sender = env.ext().caller();

				let result = <pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
					input.asset_id,
					sender,
					&input.to,
					input.value,
					true,
				);
				trace!(
					target: "runtime",
					"[ChainExtension]|call|transfer"
				);
				result.map_err(|err| {
					trace!(
						target: "runtime",
						"PSP22 Transfer failed:{:?}",
						err
					);
					DispatchError::Other("ChainExtension failed to call transfer")
				})?;

				// TODO: adjust weight
				// env.adjust_weight(charged, actual_weight)
			},

			// P2P22:transfer_from
			0x54b3c76e => {
				let mut env = env.buf_in_buf_out();

				let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
				let overhead =
					<T as pallet_contracts::Config>::Schedule::get().host_fn_weights.debug_message;
				let charged_amount = env.charge_weight(base_weight.saturating_add(overhead))?;
				trace!(
					target: "runtime",
					"[ChainExtension]|call|transfer / charge_weight:{:?}",
					charged_amount
				);

				let input: Psp22TransferFromInput<T::AssetId, T::AccountId, T::Balance> =
					env.read_as()?;
				let spender = env.ext().caller();

				let result =
					<pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::transfer_from(
						input.asset_id,
						&input.from,
						&spender,
						&input.to,
						input.value,
					);
				trace!(
					target: "runtime",
					"[ChainExtension]|call|transfer_from"
				);
				result.map_err(|err| {
					trace!(
						target: "runtime",
						"PSP22 transfer_from failed:{:?}",
						err
					);
					DispatchError::Other("ChainExtension failed to call transfer_from")
				})?;

				// TODO: adjust weight
				// env.adjust_weight(charged, actual_weight)
			},

			// PSP22::approve + PSP22::increase_allowance
			0xb20f1bbd | 0x96d6b57a => {
				let mut env = env.buf_in_buf_out();

				let base_weight = <T as pallet_assets::Config>::WeightInfo::approve_transfer();
				let overhead =
					<T as pallet_contracts::Config>::Schedule::get().host_fn_weights.debug_message;
				let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
				trace!(
					target: "runtime",
					"[ChainExtension]|call|approve / charge_weight:{:?}",
					charged_weight
				);

				let input: Psp22ApproveInput<T::AssetId, T::AccountId, T::Balance> =
					env.read_as()?;
				let owner = env.ext().caller();

				let result = <pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::approve(
					input.asset_id,
					&owner,
					&input.spender,
					input.value,
				);
				trace!(
					target: "runtime",
					"[ChainExtension]|call|approve"
				);
				result.map_err(|err| {
					trace!(
						target: "runtime",
						"PSP22 Approve failed:{:?}",
						err
					);
					DispatchError::Other("ChainExtension failed to call approve")
				})?;

				// TODO: adjust weight
				// env.adjust_weight(charged, actual_weight)
			},

			// PSP22::decrease_allowance
			0xfecb57d5 => {
				trace!(
					target: "runtime",
					"[ChainExtension]|call|decrease_allowance"
				);
				return Err(DispatchError::Other("Unimplemented func_id"))
			},

			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			},
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
