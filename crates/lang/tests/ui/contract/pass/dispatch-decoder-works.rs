use ink_lang as ink;
use ink_lang::{
    reflect::{
        ContractConstructorDecoder,
        ContractMessageDecoder,
        DecodeDispatch,
        DispatchError,
    },
    selector_bytes,
};
use scale::Encode;

#[ink::contract]
pub mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor(_input_1: bool, _input_2: i32) -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self, _input_1: bool, _input_2: i32) {}
    }
}

use contract::Contract;

fn main() {
    constructor_decoder_works();
    message_decoder_works();
}

fn constructor_decoder_works() {
    // Valid call to `constructor`:
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("constructor"));
        input_bytes.extend(true.encode());
        input_bytes.extend(42i32.encode());
        assert!(
            <<Contract as ContractConstructorDecoder>::Type as DecodeDispatch>::decode_dispatch(
                &mut &input_bytes[..]).is_ok()
        );
    }
    // Invalid call with invalid selector (or empty input).
    {
        let input_bytes = Vec::new();
        assert_eq!(
            <<Contract as ContractConstructorDecoder>::Type
                as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
                .map(|_| ())
                .unwrap_err(),
            DispatchError::InvalidSelector,
        );
    }
    // Invalid call to `message` with unknown selector.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("unknown_selector"));
        assert_eq!(
            <<Contract as ContractConstructorDecoder>::Type
                as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
                .map(|_| ())
                .unwrap_err(),
            DispatchError::UnknownSelector,
        );
    }
    // Invalid call to `message` with invalid (or missing) parameters.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("constructor"));
        assert_eq!(
            <<Contract as ContractConstructorDecoder>::Type
                as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
                .map(|_| ())
                .unwrap_err(),
            DispatchError::InvalidParameters,
        );
    }
}

fn message_decoder_works() {
    // Valid call to `message`:
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("message"));
        input_bytes.extend(true.encode());
        input_bytes.extend(42i32.encode());
        assert!(
            <<Contract as ContractMessageDecoder>::Type as DecodeDispatch>::decode_dispatch(
                &mut &input_bytes[..]).is_ok()
        );
    }
    // Invalid call with invalid selector (or empty input).
    {
        let input_bytes = Vec::new();
        assert_eq!(
            <<Contract as ContractMessageDecoder>::Type
                as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
                .map(|_| ())
                .unwrap_err(),
            DispatchError::InvalidSelector,
        );
    }
    // Invalid call to `message` with unknown selector.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("unknown_selector"));
        assert_eq!(
            <<Contract as ContractMessageDecoder>::Type
                as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
                .map(|_| ())
                .unwrap_err(),
            DispatchError::UnknownSelector,
        );
    }
    // Invalid call to `message` with invalid (or missing) parameters.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("message"));
        assert_eq!(
            <<Contract as ContractMessageDecoder>::Type
                as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
                .map(|_| ())
                .unwrap_err(),
            DispatchError::InvalidParameters,
        );
    }
}
