#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod wildcard_selector {
    #[cfg(feature = "emit-event")]
    use ink::prelude::format;
    use ink::prelude::string::String;

    #[cfg(feature = "emit-event")]
    #[ink::event]
    pub struct Event {
        pub msg: String,
    }

    #[ink(storage)]
    pub struct WildcardSelector {}

    struct MessageInput([u8; 4], String);
    impl ink::env::DecodeDispatch for MessageInput {
        fn decode_dispatch(input: &mut &[u8]) -> Result<Self, ink::env::DispatchError> {
            // todo improve code here
            let mut selector: [u8; 4] = [0u8; 4];
            selector.copy_from_slice(&input[..4]);
            let arg: String = ink::scale::Decode::decode(&mut &input[4..]).unwrap();
            Ok(MessageInput(selector, arg))
        }
    }

    impl WildcardSelector {
        /// Creates a new wildcard selector smart contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Wildcard selector handles messages with any selector.
        #[ink(message, selector = _)]
        pub fn wildcard(&mut self) {
            let MessageInput(_selector, _message) =
                ink::env::decode_input::<MessageInput>().unwrap();

            #[cfg(feature = "emit-event")]
            self.env().emit_event(Event {
                msg: format!("Wildcard selector: {_selector:?}, message: {_message}"),
            })
        }

        /// Wildcard complement handles messages with a well-known reserved selector.
        #[ink(message, selector = @)]
        pub fn wildcard_complement(&mut self, _message: String) {
            #[cfg(feature = "emit-event")]
            self.env().emit_event(Event {
                msg: format!("Wildcard complement message: {_message}"),
            });
        }
    }
}

#[cfg(test)]
mod tests;