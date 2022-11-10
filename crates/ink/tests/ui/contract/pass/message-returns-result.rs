#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, selector = 0)]
        pub fn message(&self) {}

        #[ink(message, selector = 1)]
        pub fn message_1(&self) -> Result<(), ()> {
            Ok(())
        }
    }
}

use contract::Contract;

fn main() {
    assert_eq!(
        ::core::any::TypeId::of::<
            <Contract as ::ink::reflect::DispatchableMessageInfo<
                {
                    <Contract as ::ink::reflect::ContractDispatchableMessages<
                        {
                            <Contract as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                        },
                    >>::IDS[0usize]
                },
            >>::Output,
        >(),
        ::core::any::TypeId::of::<::core::result::Result<(), ::ink::LangError>>()
    );

    assert_eq!(
        ::core::any::TypeId::of::<
            <Contract as ::ink::reflect::DispatchableMessageInfo<
                {
                    <Contract as ::ink::reflect::ContractDispatchableMessages<
                        {
                            <Contract as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                        },
                    >>::IDS[1usize]
                },
            >>::Output,
        >(),
        ::core::any::TypeId::of::<
            ::core::result::Result<::core::result::Result<(), ()>, ::ink::LangError>,
        >()
    );
}
