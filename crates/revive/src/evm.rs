use scale::{Decode, Encode};
use serde::{
    Deserialize,
    Serialize,
};
use alloc::vec::Vec;
use scale_info::TypeInfo;
use ink_primitives::{
    H160, U256, H256
};


/// A smart contract execution call trace.
#[derive(
	TypeInfo, Default, Encode, Decode, Serialize, Deserialize, Clone, Debug, Eq, PartialEq,
)]
#[serde(rename_all = "camelCase")]
pub struct CallTrace<Gas = U256> {
	/// Address of the sender.
	pub from: H160,
	/// Amount of gas provided for the call.
	pub gas: Gas,
	/// Amount of gas used.
	pub gas_used: Gas,
	/// Address of the receiver.
	pub to: H160,
	/// Call input data.
	pub input: Vec<u8>,
	/// Return data.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub output: Vec<u8>,
	/// The error message if the call failed.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub error: Option<String>,
	/// The revert reason, if the call reverted.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub revert_reason: Option<String>,
	/// List of sub-calls.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub calls: Vec<CallTrace<Gas>>,
	/// List of logs emitted during the call.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub logs: Vec<CallLog>,
	/// Amount of value transferred.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub value: Option<U256>,
	/// Type of call.
	#[serde(rename = "type")]
	pub call_type: CallType,
}

/// A log emitted during a call.
#[derive(
	Debug, Default, Clone, Encode, Decode, TypeInfo, Serialize, Deserialize, Eq, PartialEq,
)]
pub struct CallLog {
	/// The address of the contract that emitted the log.
	pub address: H160,
	/// The topics used to index the log.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub topics: Vec<H256>,
	/// The log's data.
	pub data: Vec<u8>,
	/// Position of the log relative to subcalls within the same trace
	/// See <https://github.com/ethereum/go-ethereum/pull/28389> for details
	#[serde(with = "super::hex_serde")]
	pub position: u32,
}

/// The type of call that was executed.
#[derive(
	Default, TypeInfo, Encode, Decode, Serialize, Deserialize, Eq, PartialEq, Clone, Debug,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum CallType {
	/// A regular call.
	#[default]
	Call,
	/// A read-only call.
	StaticCall,
	/// A delegate call.
	DelegateCall,
	/// A create call.
	Create,
	/// A create2 call.
	Create2,
}