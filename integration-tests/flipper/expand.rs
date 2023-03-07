#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod flipper {
    impl ::ink::env::ContractEnv for Flipper {
        type Env = ::ink::env::DefaultEnvironment;
    }
    type Environment = <Flipper as ::ink::env::ContractEnv>::Env;
    type AccountId = <<Flipper as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::AccountId;
    type Balance = <<Flipper as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Balance;
    type Hash = <<Flipper as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Hash;
    type Timestamp = <<Flipper as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Timestamp;
    type BlockNumber = <<Flipper as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::BlockNumber;
    const _: () = {
        struct Check {
            salt: (),
            field_0: bool,
        }
    };
    #[cfg(not(feature = "__ink_dylint_Storage"))]
    pub struct Flipper {
        value: <bool as ::ink::storage::traits::AutoStorableHint<
            ::ink::storage::traits::ManualKey<2054318728u32, ()>,
        >>::Type,
    }
    const _: () = {
        impl<
            __ink_generic_salt: ::ink::storage::traits::StorageKey,
        > ::ink::storage::traits::StorableHint<__ink_generic_salt> for Flipper {
            type Type = Flipper;
            type PreferredKey = ::ink::storage::traits::AutoKey;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageKey for Flipper {
            const KEY: ::ink::primitives::Key = <() as ::ink::storage::traits::StorageKey>::KEY;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::Storable for Flipper {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::scale::Input>(
                __input: &mut __ink_I,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(Flipper {
                    value: <<bool as ::ink::storage::traits::AutoStorableHint<
                        ::ink::storage::traits::ManualKey<2054318728u32, ()>,
                    >>::Type as ::ink::storage::traits::Storable>::decode(__input)?,
                })
            }
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __dest: &mut __ink_O,
            ) {
                match self {
                    Flipper { value: __binding_0 } => {
                        ::ink::storage::traits::Storable::encode(__binding_0, __dest);
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for Flipper {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("Flipper", "flipper::flipper"))
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <bool as ::ink::storage::traits::AutoStorableHint<
                                            ::ink::storage::traits::ManualKey<2054318728u32, ()>,
                                        >>::Type,
                                    >()
                                    .name("value")
                                    .type_name(
                                        "<bool as::ink::storage::traits::AutoStorableHint<::ink::storage\n::traits::ManualKey<2054318728u32, ()>,>>::Type",
                                    )
                            }),
                    )
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for Flipper {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "Flipper",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "value",
                                <<bool as ::ink::storage::traits::AutoStorableHint<
                                    ::ink::storage::traits::ManualKey<2054318728u32, ()>,
                                >>::Type as ::ink::storage::traits::StorageLayout>::layout(
                                    __key,
                                ),
                            ),
                        ],
                    ),
                )
            }
        }
    };
    const _: () = {
        impl ::ink::reflect::ContractName for Flipper {
            const NAME: &'static str = "Flipper";
        }
    };
    const _: () = {
        impl<'a> ::ink::codegen::Env for &'a Flipper {
            type EnvAccess = ::ink::EnvAccess<
                'a,
                <Flipper as ::ink::env::ContractEnv>::Env,
            >;
            fn env(self) -> Self::EnvAccess {
                <<Self as ::ink::codegen::Env>::EnvAccess as ::core::default::Default>::default()
            }
        }
        impl<'a> ::ink::codegen::StaticEnv for Flipper {
            type EnvAccess = ::ink::EnvAccess<
                'static,
                <Flipper as ::ink::env::ContractEnv>::Env,
            >;
            fn env() -> Self::EnvAccess {
                <<Self as ::ink::codegen::StaticEnv>::EnvAccess as ::core::default::Default>::default()
            }
        }
    };
    const _: () = {
        #[allow(unused_imports)]
        use ::ink::codegen::{Env as _, StaticEnv as _};
    };
    impl ::ink::reflect::ContractAmountDispatchables for Flipper {
        const CONSTRUCTORS: ::core::primitive::usize = {
            let mut count: usize = 2usize;
            count
        };
    }
    impl ::ink::reflect::ContractDispatchableConstructors<
        { <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS },
    > for Flipper {
        const IDS: [::core::primitive::u32; <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS] = [
            0x9BAE9D5E_u32,
            0x61EF7E3E_u32,
        ];
    }
    impl ::ink::reflect::DispatchableConstructorInfo<0x9BAE9D5E_u32> for Flipper {
        type Input = bool;
        type Output = Self;
        type Storage = Flipper;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Flipper>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Flipper>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |__ink_binding_0| {
            Flipper::new(__ink_binding_0)
        };
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x9B_u8,
            0xAE_u8,
            0x9D_u8,
            0x5E_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new";
    }
    impl ::ink::reflect::DispatchableConstructorInfo<0x61EF7E3E_u32> for Flipper {
        type Input = ();
        type Output = Self;
        type Storage = Flipper;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Flipper>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Flipper>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |_| { Flipper::new_default() };
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x61_u8,
            0xEF_u8,
            0x7E_u8,
            0x3E_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new_default";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x633AA551_u32> for Flipper {
        type Input = ();
        type Output = ();
        type Storage = Flipper;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { Flipper::flip(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x63_u8,
            0x3A_u8,
            0xA5_u8,
            0x51_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "flip";
    }
    #[cfg(feature = "foo")]
    impl ::ink::reflect::DispatchableMessageInfo<0x8212E40D_u32> for Flipper {
        type Input = bool;
        type Output = ();
        type Storage = Flipper;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { Flipper::push_foo(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x82_u8,
            0x12_u8,
            0xE4_u8,
            0x0D_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "push_foo";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x2F865BD9_u32> for Flipper {
        type Input = ();
        type Output = bool;
        type Storage = Flipper;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { Flipper::get(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x2F_u8,
            0x86_u8,
            0x5B_u8,
            0xD9_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "get";
    }
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_ConstructorDecoder {
            Constructor0(
                <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                            },
                        >>::IDS[0usize]
                    },
                >>::Input,
            ),
            Constructor1(
                <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                            },
                        >>::IDS[1usize]
                    },
                >>::Input,
            ),
        }
        impl ::ink::reflect::DecodeDispatch for __ink_ConstructorDecoder {
            fn decode_dispatch<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
            where
                I: ::scale::Input,
            {
                const CONSTRUCTOR_0: [::core::primitive::u8; 4usize] = <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                            },
                        >>::IDS[0usize]
                    },
                >>::SELECTOR;
                const CONSTRUCTOR_1: [::core::primitive::u8; 4usize] = <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                            },
                        >>::IDS[1usize]
                    },
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    CONSTRUCTOR_0 => {
                        ::core::result::Result::Ok(
                            Self::Constructor0(
                                <<Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    CONSTRUCTOR_1 => {
                        ::core::result::Result::Ok(
                            Self::Constructor1(
                                <<Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    _invalid => {
                        ::core::result::Result::Err(
                            ::ink::reflect::DispatchError::UnknownSelector,
                        )
                    }
                }
            }
        }
        impl ::scale::Decode for __ink_ConstructorDecoder {
            fn decode<I>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error>
            where
                I: ::scale::Input,
            {
                <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                    .map_err(::core::convert::Into::into)
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_ConstructorDecoder {
            #[allow(clippy::nonminimal_bool)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                match self {
                    Self::Constructor0(input) => {
                        if {
                            false
                                || <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::PAYABLE
                                || <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::PAYABLE
                        }
                            && !<Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[0usize]
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Flipper as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[0usize]
                            },
                        >>::Output = <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[0usize]
                            },
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[0usize]
                                },
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            Flipper,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                Flipper,
                            >(
                                &<Flipper as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                            {
                                                <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                                    {
                                                        <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                                    },
                                                >>::IDS[0usize]
                                            },
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<Flipper>>::Error,
                                >,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(
                                output_result.is_err(),
                            ),
                            &::ink::ConstructorResult::Ok(output_result.map(|_| ())),
                        );
                    }
                    Self::Constructor1(input) => {
                        if {
                            false
                                || <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::PAYABLE
                                || <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::PAYABLE
                        }
                            && !<Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[1usize]
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Flipper as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[1usize]
                            },
                        >>::Output = <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[1usize]
                            },
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[1usize]
                                },
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            Flipper,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                Flipper,
                            >(
                                &<Flipper as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                            {
                                                <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                                    {
                                                        <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                                    },
                                                >>::IDS[1usize]
                                            },
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<Flipper>>::Error,
                                >,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(
                                output_result.is_err(),
                            ),
                            &::ink::ConstructorResult::Ok(output_result.map(|_| ())),
                        );
                    }
                }
            }
        }
        impl ::ink::reflect::ContractConstructorDecoder for Flipper {
            type Type = __ink_ConstructorDecoder;
        }
    };
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_MessageDecoder {
            Message0(
                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                    0x633AA551_u32,
                >>::Input,
            ),
            #[cfg(feature = "foo")]
            Message1(
                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                    0x8212E40D_u32,
                >>::Input,
            ),
            Message2(
                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                    0x2F865BD9_u32,
                >>::Input,
            ),
        }
        impl ::ink::reflect::DecodeDispatch for __ink_MessageDecoder {
            fn decode_dispatch<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
            where
                I: ::scale::Input,
            {
                const MESSAGE_0: [::core::primitive::u8; 4usize] = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                    { 0x633AA551_u32 },
                >>::SELECTOR;
                #[cfg(feature = "foo")]
                const MESSAGE_1: [::core::primitive::u8; 4usize] = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                    { 0x8212E40D_u32 },
                >>::SELECTOR;
                const MESSAGE_2: [::core::primitive::u8; 4usize] = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                    { 0x2F865BD9_u32 },
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    MESSAGE_0 => {
                        ::core::result::Result::Ok(
                            Self::Message0(
                                <<Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    0x633AA551_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    #[cfg(feature = "foo")]
                    MESSAGE_1 => {
                        ::core::result::Result::Ok(
                            Self::Message1(
                                <<Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    0x8212E40D_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_2 => {
                        ::core::result::Result::Ok(
                            Self::Message2(
                                <<Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    0x2F865BD9_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    _invalid => {
                        ::core::result::Result::Err(
                            ::ink::reflect::DispatchError::UnknownSelector,
                        )
                    }
                }
            }
        }
        impl ::scale::Decode for __ink_MessageDecoder {
            fn decode<I>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error>
            where
                I: ::scale::Input,
            {
                <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                    .map_err(::core::convert::Into::into)
            }
        }
        fn push_contract(contract: ::core::mem::ManuallyDrop<Flipper>, mutates: bool) {
            if mutates {
                ::ink::env::set_contract_storage::<
                    ::ink::primitives::Key,
                    Flipper,
                >(&<Flipper as ::ink::storage::traits::StorageKey>::KEY, &contract);
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_MessageDecoder {
            #[allow(clippy::nonminimal_bool, clippy::let_unit_value)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                let key = <Flipper as ::ink::storage::traits::StorageKey>::KEY;
                let mut contract: ::core::mem::ManuallyDrop<Flipper> = ::core::mem::ManuallyDrop::new(
                    match ::ink::env::get_contract_storage(&key) {
                        ::core::result::Result::Ok(
                            ::core::option::Option::Some(value),
                        ) => value,
                        ::core::result::Result::Ok(::core::option::Option::None) => {
                            ::core::panicking::panic_fmt(
                                format_args!("storage entry was empty"),
                            )
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::panicking::panic_fmt(
                                format_args!("could not properly decode storage entry"),
                            )
                        }
                    },
                );
                match self {
                    Self::Message0(input) => {
                        let message_0 = false;
                        let message_1 = false;
                        let message_2 = false;
                        let message_0 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x633AA551_u32 },
                        >>::PAYABLE;
                        #[cfg(feature = "foo")]
                        let message_1 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x8212E40D_u32 },
                        >>::PAYABLE;
                        let message_2 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x2F865BD9_u32 },
                        >>::PAYABLE;
                        if (false || message_0 || message_1 || message_2)
                            && !<Flipper as ::ink::reflect::DispatchableMessageInfo<
                                { 0x633AA551_u32 },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Flipper as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x633AA551_u32 },
                        >>::Output = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x633AA551_u32 },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x633AA551_u32 },
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        if !is_reverted {
                            push_contract(
                                contract,
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x633AA551_u32 },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x633AA551_u32 },
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    #[cfg(feature = "foo")]
                    Self::Message1(input) => {
                        let message_0 = false;
                        let message_1 = false;
                        let message_2 = false;
                        let message_0 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x633AA551_u32 },
                        >>::PAYABLE;
                        #[cfg(feature = "foo")]
                        let message_1 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x8212E40D_u32 },
                        >>::PAYABLE;
                        let message_2 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x2F865BD9_u32 },
                        >>::PAYABLE;
                        if (false || message_0 || message_1 || message_2)
                            && !<Flipper as ::ink::reflect::DispatchableMessageInfo<
                                { 0x8212E40D_u32 },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Flipper as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x8212E40D_u32 },
                        >>::Output = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x8212E40D_u32 },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x8212E40D_u32 },
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        if !is_reverted {
                            push_contract(
                                contract,
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x8212E40D_u32 },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x8212E40D_u32 },
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    Self::Message2(input) => {
                        let message_0 = false;
                        let message_1 = false;
                        let message_2 = false;
                        let message_0 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x633AA551_u32 },
                        >>::PAYABLE;
                        #[cfg(feature = "foo")]
                        let message_1 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x8212E40D_u32 },
                        >>::PAYABLE;
                        let message_2 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x2F865BD9_u32 },
                        >>::PAYABLE;
                        if (false || message_0 || message_1 || message_2)
                            && !<Flipper as ::ink::reflect::DispatchableMessageInfo<
                                { 0x2F865BD9_u32 },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Flipper as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x2F865BD9_u32 },
                        >>::Output = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                            { 0x2F865BD9_u32 },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x2F865BD9_u32 },
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        if !is_reverted {
                            push_contract(
                                contract,
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x2F865BD9_u32 },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Flipper as ::ink::reflect::DispatchableMessageInfo<
                                    { 0x2F865BD9_u32 },
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                };
            }
        }
        impl ::ink::reflect::ContractMessageDecoder for Flipper {
            type Type = __ink_MessageDecoder;
        }
    };
    #[cfg(not(test))]
    #[cfg(not(feature = "ink-as-dependency"))]
    const _: () = {
        #[cfg(not(test))]
        #[no_mangle]
        #[allow(clippy::nonminimal_bool)]
        fn deploy() {
            if !{
                false
                    || <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                        {
                            <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                {
                                    <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                },
                            >>::IDS[0usize]
                        },
                    >>::PAYABLE
                    || <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                        {
                            <Flipper as ::ink::reflect::ContractDispatchableConstructors<
                                {
                                    <Flipper as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                },
                            >>::IDS[1usize]
                        },
                    >>::PAYABLE
            } {
                ::ink::codegen::deny_payment::<
                    <Flipper as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| ::core::panicking::panic_display(&error))
            }
            let dispatchable = match ::ink::env::decode_input::<
                <Flipper as ::ink::reflect::ContractConstructorDecoder>::Type,
            >() {
                ::core::result::Result::Ok(decoded_dispatchable) => decoded_dispatchable,
                ::core::result::Result::Err(_decoding_error) => {
                    let error = ::ink::ConstructorResult::Err(
                        ::ink::LangError::CouldNotReadInput,
                    );
                    ::ink::env::return_value::<
                        ::ink::ConstructorResult<()>,
                    >(::ink::env::ReturnFlags::new_with_reverted(true), &error);
                }
            };
            <<Flipper as ::ink::reflect::ContractConstructorDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
                    dispatchable,
                )
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!("dispatching ink! message failed: {0}", error),
                    )
                })
        }
        #[cfg(not(test))]
        #[no_mangle]
        #[allow(clippy::nonminimal_bool)]
        fn call() {
            let message_0 = false;
            let message_1 = false;
            let message_2 = false;
            let message_0 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                { 0x633AA551_u32 },
            >>::PAYABLE;
            #[cfg(feature = "foo")]
            let message_1 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                { 0x8212E40D_u32 },
            >>::PAYABLE;
            let message_2 = <Flipper as ::ink::reflect::DispatchableMessageInfo<
                { 0x2F865BD9_u32 },
            >>::PAYABLE;
            if !(false || message_0 || message_1 || message_2) {
                ::ink::codegen::deny_payment::<
                    <Flipper as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| ::core::panicking::panic_display(&error))
            }
            let dispatchable = match ::ink::env::decode_input::<
                <Flipper as ::ink::reflect::ContractMessageDecoder>::Type,
            >() {
                ::core::result::Result::Ok(decoded_dispatchable) => decoded_dispatchable,
                ::core::result::Result::Err(_decoding_error) => {
                    let error = ::ink::MessageResult::Err(
                        ::ink::LangError::CouldNotReadInput,
                    );
                    ::ink::env::return_value::<
                        ::ink::MessageResult<()>,
                    >(::ink::env::ReturnFlags::new_with_reverted(true), &error);
                }
            };
            <<Flipper as ::ink::reflect::ContractMessageDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
                    dispatchable,
                )
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!("dispatching ink! message failed: {0}", error),
                    )
                })
        }
    };
    const _: () = {
        use ::ink::codegen::{Env as _, StaticEnv as _};
        const _: ::ink::codegen::utils::IsSameType<Flipper> = ::ink::codegen::utils::IsSameType::<
            Flipper,
        >::new();
        impl Flipper {
            /// Creates a new flipper smart contract initialized with the given value.
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new(init_value: bool) -> Self {
                Self { value: init_value }
            }
            /// Creates a new flipper smart contract initialized to `false`.
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new_default() -> Self {
                Self::new(Default::default())
            }
            /// Flips the current value of the Flipper's boolean.
            pub fn flip(&mut self) {
                self.value = !self.value;
            }
            /// Returns the current value of the Flipper's boolean.
            #[cfg(feature = "foo")]
            pub fn push_foo(&mut self, value: bool) {
                self.value = value;
            }
            /// Returns the current value of the Flipper's boolean.
            pub fn get(&self) -> bool {
                self.value
            }
        }
        const _: () = {
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<bool>,
            >();
        };
    };
    const _: () = {
        /// The ink! smart contract's call builder.
        ///
        /// Implements the underlying on-chain calling of the ink! smart contract
        /// messages and trait implementations in a type safe way.
        #[repr(transparent)]
        pub struct CallBuilder {
            account_id: AccountId,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CallBuilder {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "CallBuilder",
                    "account_id",
                    &&self.account_id,
                )
            }
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for CallBuilder {
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    ::scale::Encode::encode_to(&&self.account_id, __codec_dest_edqy)
                }
                fn encode(&self) -> ::scale::alloc::vec::Vec<::core::primitive::u8> {
                    ::scale::Encode::encode(&&self.account_id)
                }
                fn using_encoded<
                    R,
                    F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R,
                >(&self, f: F) -> R {
                    ::scale::Encode::using_encoded(&&self.account_id, f)
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for CallBuilder {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for CallBuilder {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    ::core::result::Result::Ok(CallBuilder {
                        account_id: {
                            let __codec_res_edqy = <AccountId as ::scale::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `CallBuilder::account_id`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::hash::Hash for CallBuilder {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.account_id, state)
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CallBuilder {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CallBuilder {
            #[inline]
            fn eq(&self, other: &CallBuilder) -> bool {
                self.account_id == other.account_id
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralEq for CallBuilder {}
        #[automatically_derived]
        impl ::core::cmp::Eq for CallBuilder {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<AccountId>;
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CallBuilder {
            #[inline]
            fn clone(&self) -> CallBuilder {
                CallBuilder {
                    account_id: ::core::clone::Clone::clone(&self.account_id),
                }
            }
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for CallBuilder {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(::scale_info::Path::new("CallBuilder", "flipper::flipper"))
                        .type_params(::alloc::vec::Vec::new())
                        .docs(
                            &[
                                "The ink! smart contract's call builder.",
                                "",
                                "Implements the underlying on-chain calling of the ink! smart contract",
                                "messages and trait implementations in a type safe way.",
                            ],
                        )
                        .composite(
                            ::scale_info::build::Fields::named()
                                .field(|f| {
                                    f
                                        .ty::<AccountId>()
                                        .name("account_id")
                                        .type_name("AccountId")
                                }),
                        )
                }
            }
        };
        const _: () = {
            impl ::ink::storage::traits::StorageLayout for CallBuilder {
                fn layout(
                    __key: &::ink::primitives::Key,
                ) -> ::ink::metadata::layout::Layout {
                    ::ink::metadata::layout::Layout::Struct(
                        ::ink::metadata::layout::StructLayout::new(
                            "CallBuilder",
                            [
                                ::ink::metadata::layout::FieldLayout::new(
                                    "account_id",
                                    <AccountId as ::ink::storage::traits::StorageLayout>::layout(
                                        __key,
                                    ),
                                ),
                            ],
                        ),
                    )
                }
            }
        };
        const _: () = {
            impl ::ink::codegen::ContractCallBuilder for Flipper {
                type Type = CallBuilder;
            }
            impl ::ink::env::ContractEnv for CallBuilder {
                type Env = <Flipper as ::ink::env::ContractEnv>::Env;
            }
        };
        impl ::ink::env::call::FromAccountId<Environment> for CallBuilder {
            #[inline]
            fn from_account_id(account_id: AccountId) -> Self {
                Self { account_id }
            }
        }
        impl ::ink::ToAccountId<Environment> for CallBuilder {
            #[inline]
            fn to_account_id(&self) -> AccountId {
                <AccountId as ::core::clone::Clone>::clone(&self.account_id)
            }
        }
        impl ::core::convert::AsRef<AccountId> for CallBuilder {
            fn as_ref(&self) -> &AccountId {
                &self.account_id
            }
        }
        impl ::core::convert::AsMut<AccountId> for CallBuilder {
            fn as_mut(&mut self) -> &mut AccountId {
                &mut self.account_id
            }
        }
        impl CallBuilder {
            /// Flips the current value of the Flipper's boolean.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn flip(
                &mut self,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0x63_u8,
                                0x3A_u8,
                                0xA5_u8,
                                0x51_u8,
                            ]),
                        ),
                    )
                    .returns::<()>()
            }
            /// Returns the current value of the Flipper's boolean.
            #[cfg(feature = "foo")]
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn push_foo(
                &mut self,
                __ink_binding_0: bool,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<bool>,
                            ::ink::env::call::utils::EmptyArgumentList,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0x82_u8,
                                    0x12_u8,
                                    0xE4_u8,
                                    0x0D_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<()>()
            }
            /// Returns the current value of the Flipper's boolean.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn get(
                &self,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<bool>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0x2F_u8,
                                0x86_u8,
                                0x5B_u8,
                                0xD9_u8,
                            ]),
                        ),
                    )
                    .returns::<bool>()
            }
        }
    };
    pub struct FlipperRef {
        inner: <Flipper as ::ink::codegen::ContractCallBuilder>::Type,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for FlipperRef {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "FlipperRef",
                "inner",
                &&self.inner,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for FlipperRef {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::scale::Encode::encode_to(&&self.inner, __codec_dest_edqy)
            }
            fn encode(&self) -> ::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::scale::Encode::encode(&&self.inner)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::scale::Encode::using_encoded(&&self.inner, f)
            }
        }
        #[automatically_derived]
        impl ::scale::EncodeLike for FlipperRef {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Decode for FlipperRef {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(FlipperRef {
                    inner: {
                        let __codec_res_edqy = <<Flipper as ::ink::codegen::ContractCallBuilder>::Type as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `FlipperRef::inner`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::hash::Hash for FlipperRef {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.inner, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for FlipperRef {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for FlipperRef {
        #[inline]
        fn eq(&self, other: &FlipperRef) -> bool {
            self.inner == other.inner
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for FlipperRef {}
    #[automatically_derived]
    impl ::core::cmp::Eq for FlipperRef {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<
                <Flipper as ::ink::codegen::ContractCallBuilder>::Type,
            >;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for FlipperRef {
        #[inline]
        fn clone(&self) -> FlipperRef {
            FlipperRef {
                inner: ::core::clone::Clone::clone(&self.inner),
            }
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for FlipperRef {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new("FlipperRef", "flipper::flipper"))
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <Flipper as ::ink::codegen::ContractCallBuilder>::Type,
                                    >()
                                    .name("inner")
                                    .type_name(
                                        "<Flipper as::ink::codegen::ContractCallBuilder>::Type",
                                    )
                            }),
                    )
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for FlipperRef {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "FlipperRef",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "inner",
                                <<Flipper as ::ink::codegen::ContractCallBuilder>::Type as ::ink::storage::traits::StorageLayout>::layout(
                                    __key,
                                ),
                            ),
                        ],
                    ),
                )
            }
        }
    };
    const _: () = {
        impl ::ink::env::ContractReference for Flipper {
            type Type = FlipperRef;
        }
        impl ::ink::env::call::ConstructorReturnType<FlipperRef> for Flipper {
            type Output = FlipperRef;
            type Error = ();
            fn ok(value: FlipperRef) -> Self::Output {
                value
            }
        }
        impl<E> ::ink::env::call::ConstructorReturnType<FlipperRef>
        for ::core::result::Result<Flipper, E>
        where
            E: ::scale::Decode,
        {
            const IS_RESULT: bool = true;
            type Output = ::core::result::Result<FlipperRef, E>;
            type Error = E;
            fn ok(value: FlipperRef) -> Self::Output {
                ::core::result::Result::Ok(value)
            }
            fn err(err: Self::Error) -> ::core::option::Option<Self::Output> {
                ::core::option::Option::Some(::core::result::Result::Err(err))
            }
        }
        impl ::ink::env::ContractEnv for FlipperRef {
            type Env = <Flipper as ::ink::env::ContractEnv>::Env;
        }
    };
    impl FlipperRef {
        /// Creates a new flipper smart contract initialized with the given value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new(
            __ink_binding_0: bool,
        ) -> ::ink::env::call::CreateBuilder<
            Environment,
            Self,
            ::ink::env::call::utils::Unset<Hash>,
            ::ink::env::call::utils::Unset<u64>,
            ::ink::env::call::utils::Unset<Balance>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::ArgumentList<
                        ::ink::env::call::utils::Argument<bool>,
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
            >,
            ::ink::env::call::utils::Unset<::ink::env::call::state::Salt>,
            ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<Self>>,
        > {
            ::ink::env::call::build_create::<Self>()
                .exec_input(
                    ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0x9B_u8,
                                0xAE_u8,
                                0x9D_u8,
                                0x5E_u8,
                            ]),
                        )
                        .push_arg(__ink_binding_0),
                )
                .returns::<Self>()
        }
        /// Creates a new flipper smart contract initialized to `false`.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new_default() -> ::ink::env::call::CreateBuilder<
            Environment,
            Self,
            ::ink::env::call::utils::Unset<Hash>,
            ::ink::env::call::utils::Unset<u64>,
            ::ink::env::call::utils::Unset<Balance>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::EmptyArgumentList,
                >,
            >,
            ::ink::env::call::utils::Unset<::ink::env::call::state::Salt>,
            ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<Self>>,
        > {
            ::ink::env::call::build_create::<Self>()
                .exec_input(
                    ::ink::env::call::ExecutionInput::new(
                        ::ink::env::call::Selector::new([
                            0x61_u8,
                            0xEF_u8,
                            0x7E_u8,
                            0x3E_u8,
                        ]),
                    ),
                )
                .returns::<Self>()
        }
        /// Flips the current value of the Flipper's boolean.
        #[inline]
        pub fn flip(&mut self) {
            self.try_flip()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}", "Flipper",
                        "flip", error
                    ),
                ))
        }
        /// Flips the current value of the Flipper's boolean.
        #[inline]
        pub fn try_flip(&mut self) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .flip()
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}", "Flipper",
                        "flip", error
                    ),
                ))
        }
        /// Returns the current value of the Flipper's boolean.
        #[cfg(feature = "foo")]
        #[inline]
        pub fn push_foo(&mut self, value: bool) {
            self.try_push_foo(value)
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}", "Flipper",
                        "push_foo", error
                    ),
                ))
        }
        /// Returns the current value of the Flipper's boolean.
        #[cfg(feature = "foo")]
        #[inline]
        pub fn try_push_foo(&mut self, value: bool) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .push_foo(value)
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}", "Flipper",
                        "push_foo", error
                    ),
                ))
        }
        /// Returns the current value of the Flipper's boolean.
        #[inline]
        pub fn get(&self) -> bool {
            self.try_get()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}", "Flipper",
                        "get", error
                    ),
                ))
        }
        /// Returns the current value of the Flipper's boolean.
        #[inline]
        pub fn try_get(&self) -> ::ink::MessageResult<bool> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .get()
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}", "Flipper",
                        "get", error
                    ),
                ))
        }
    }
    const _: () = {
        impl ::ink::codegen::TraitCallBuilder for FlipperRef {
            type Builder = <Flipper as ::ink::codegen::ContractCallBuilder>::Type;
            #[inline]
            fn call(&self) -> &Self::Builder {
                &self.inner
            }
            #[inline]
            fn call_mut(&mut self) -> &mut Self::Builder {
                &mut self.inner
            }
        }
    };
    impl ::ink::env::call::FromAccountId<Environment> for FlipperRef {
        #[inline]
        fn from_account_id(account_id: AccountId) -> Self {
            Self {
                inner: <<Flipper as ::ink::codegen::ContractCallBuilder>::Type as ::ink::env::call::FromAccountId<
                    Environment,
                >>::from_account_id(account_id),
            }
        }
    }
    impl ::ink::ToAccountId<Environment> for FlipperRef {
        #[inline]
        fn to_account_id(&self) -> AccountId {
            <<Flipper as ::ink::codegen::ContractCallBuilder>::Type as ::ink::ToAccountId<
                Environment,
            >>::to_account_id(&self.inner)
        }
    }
    impl ::core::convert::AsRef<AccountId> for FlipperRef {
        fn as_ref(&self) -> &AccountId {
            <_ as ::core::convert::AsRef<AccountId>>::as_ref(&self.inner)
        }
    }
    impl ::core::convert::AsMut<AccountId> for FlipperRef {
        fn as_mut(&mut self) -> &mut AccountId {
            <_ as ::core::convert::AsMut<AccountId>>::as_mut(&mut self.inner)
        }
    }
    #[cfg(feature = "std")]
    #[cfg(not(feature = "ink-as-dependency"))]
    const _: () = {
        #[no_mangle]
        pub fn __ink_generate_metadata() -> ::ink::metadata::InkProject {
            let layout = ::ink::metadata::layout::Layout::Root(
                ::ink::metadata::layout::RootLayout::new(
                    <::ink::metadata::layout::LayoutKey as ::core::convert::From<
                        ::ink::primitives::Key,
                    >>::from(<Flipper as ::ink::storage::traits::StorageKey>::KEY),
                    <Flipper as ::ink::storage::traits::StorageLayout>::layout(
                        &<Flipper as ::ink::storage::traits::StorageKey>::KEY,
                    ),
                ),
            );
            ::ink::metadata::layout::ValidateLayout::validate(&layout)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!("metadata ink! generation failed: {0}", error),
                    )
                });
            ::ink::metadata::InkProject::new(
                layout,
                ::ink::metadata::ContractSpec::new()
                    .constructors([
                        ::ink::metadata::ConstructorSpec::from_label("new")
                            .selector([0x9B_u8, 0xAE_u8, 0x9D_u8, 0x5E_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("init_value")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            bool,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["bool"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                        2611912030u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                                            2611912030u32,
                                                        >>::Error,
                                                    >,
                                                >,
                                            >("ink_primitives::ConstructorResult"),
                                        )
                                    } else {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<()>,
                                            >("ink_primitives::ConstructorResult"),
                                        )
                                    },
                                ),
                            )
                            .docs([
                                " Creates a new flipper smart contract initialized with the given value.",
                            ])
                            .done(),
                        ::ink::metadata::ConstructorSpec::from_label("new_default")
                            .selector([0x61_u8, 0xEF_u8, 0x7E_u8, 0x3E_u8])
                            .args([])
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                        1643085374u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <Flipper as ::ink::reflect::DispatchableConstructorInfo<
                                                            1643085374u32,
                                                        >>::Error,
                                                    >,
                                                >,
                                            >("ink_primitives::ConstructorResult"),
                                        )
                                    } else {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<()>,
                                            >("ink_primitives::ConstructorResult"),
                                        )
                                    },
                                ),
                            )
                            .docs([
                                " Creates a new flipper smart contract initialized to `false`.",
                            ])
                            .done(),
                    ])
                    .messages([
                        ::ink::metadata::MessageSpec::from_label("flip")
                            .selector([0x63_u8, 0x3A_u8, 0xA5_u8, 0x51_u8])
                            .args([])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<()>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(true)
                            .payable(false)
                            .docs([" Flips the current value of the Flipper's boolean."])
                            .done(),
                        #[cfg(feature = "foo")]
                        ::ink::metadata::MessageSpec::from_label("push_foo")
                            .selector([0x82_u8, 0x12_u8, 0xE4_u8, 0x0D_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("value")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            bool,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["bool"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<()>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(true)
                            .payable(false)
                            .docs([
                                " Returns the current value of the Flipper's boolean.",
                            ])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("get")
                            .selector([0x2F_u8, 0x86_u8, 0x5B_u8, 0xD9_u8])
                            .args([])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<bool>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(false)
                            .payable(false)
                            .docs([
                                " Returns the current value of the Flipper's boolean.",
                            ])
                            .done(),
                    ])
                    .events([])
                    .docs([])
                    .lang_error(
                        ::ink::metadata::TypeSpec::with_name_segs::<
                            ::ink::LangError,
                            _,
                        >(
                            ::core::iter::Iterator::map(
                                ::core::iter::IntoIterator::into_iter(["ink", "LangError"]),
                                ::core::convert::AsRef::as_ref,
                            ),
                        ),
                    )
                    .done(),
            )
        }
    };
}
