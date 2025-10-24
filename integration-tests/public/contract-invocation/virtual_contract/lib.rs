#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::virtual_contract::VirtualContractRef;

#[ink::contract]
pub mod virtual_contract {
    use ink::env::call::{
        ExecutionInput,
        Selector,
        build_call,
    };

    #[ink(storage)]
    pub struct VirtualContract {
        version: Address,
        x: u32,
    }

    impl VirtualContract {
        /// Creates a new Template contract.
        #[ink(constructor)]
        pub fn new(version: Address, x: u32) -> Self {
            Self { version, x }
        }

        #[ink(message)]
        pub fn set_version(&mut self, version: Address) {
            self.version = version;
        }

        #[ink(message)]
        pub fn real_set_x(&mut self, x: u32) {
            self.x = x;
        }

        #[ink(message)]
        pub fn real_get_x(&self) -> u32 {
            self.x
        }

        #[ink(message)]
        pub fn set_x(&mut self, x: u32) {
            let call = build_call()
                .delegate(self.version)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("set_x")))
                        .push_arg(x),
                )
                .returns::<()>()
                .params();

            self.env()
                .invoke_contract_delegate(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| {
                    panic!("Received a `LangError`: {lang_err:?}")
                });
        }

        #[ink(message)]
        pub fn get_x(&self) -> u32 {
            let call = build_call()
                .delegate(self.version)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get_x"
                ))))
                .returns::<u32>()
                .params();

            self.env()
                .invoke_contract_delegate(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }
    }

    impl Default for VirtualContract {
        fn default() -> Self {
            Self::new(Address::default(), 0)
        }
    }
}
