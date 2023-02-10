#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod quickcheck_tests {
    impl ::ink::env::ContractEnv for QuickcheckTests {
        type Env = ::ink::env::DefaultEnvironment;
    }
    type Environment = <QuickcheckTests as ::ink::env::ContractEnv>::Env;
    type AccountId = <<QuickcheckTests as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::AccountId;
    type Balance = <<QuickcheckTests as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Balance;
    type Hash = <<QuickcheckTests as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Hash;
    type Timestamp = <<QuickcheckTests as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Timestamp;
    type BlockNumber = <<QuickcheckTests as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::BlockNumber;
    const _: () = {
        struct Check {
            salt: (),
            field_0: i32,
        }
    };
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[cfg(not(feature = "__ink_dylint_Storage"))]
    pub struct QuickcheckTests {
        /// Stores a single `bool` value on the storage.
        value: <i32 as ::ink::storage::traits::AutoStorableHint<
            ::ink::storage::traits::ManualKey<2507150569u32, ()>,
        >>::Type,
    }
    const _: () = {
        impl<
            __ink_generic_salt: ::ink::storage::traits::StorageKey,
        > ::ink::storage::traits::StorableHint<__ink_generic_salt> for QuickcheckTests {
            type Type = QuickcheckTests;
            type PreferredKey = ::ink::storage::traits::AutoKey;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageKey for QuickcheckTests {
            const KEY: ::ink::primitives::Key = <() as ::ink::storage::traits::StorageKey>::KEY;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::Storable for QuickcheckTests {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::scale::Input>(
                __input: &mut __ink_I,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(QuickcheckTests {
                    value: <<i32 as ::ink::storage::traits::AutoStorableHint<
                        ::ink::storage::traits::ManualKey<2507150569u32, ()>,
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
                    QuickcheckTests { value: __binding_0 } => {
                        ::ink::storage::traits::Storable::encode(__binding_0, __dest);
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for QuickcheckTests {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new(
                            "QuickcheckTests",
                            "quickcheck_tests::quickcheck_tests",
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .docs(
                        &[
                            "Defines the storage of your contract.",
                            "Add new fields to the below struct in order",
                            "to add new static storage fields to your contract.",
                        ],
                    )
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <i32 as ::ink::storage::traits::AutoStorableHint<
                                            ::ink::storage::traits::ManualKey<2507150569u32, ()>,
                                        >>::Type,
                                    >()
                                    .name("value")
                                    .type_name(
                                        "<i32 as::ink::storage::traits::AutoStorableHint<::ink::storage\n::traits::ManualKey<2507150569u32, ()>,>>::Type",
                                    )
                                    .docs(&["Stores a single `bool` value on the storage."])
                            }),
                    )
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for QuickcheckTests {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "QuickcheckTests",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "value",
                                <<i32 as ::ink::storage::traits::AutoStorableHint<
                                    ::ink::storage::traits::ManualKey<2507150569u32, ()>,
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
        impl ::ink::reflect::ContractName for QuickcheckTests {
            const NAME: &'static str = "QuickcheckTests";
        }
    };
    const _: () = {
        impl<'a> ::ink::codegen::Env for &'a QuickcheckTests {
            type EnvAccess = ::ink::EnvAccess<
                'a,
                <QuickcheckTests as ::ink::env::ContractEnv>::Env,
            >;
            fn env(self) -> Self::EnvAccess {
                <<Self as ::ink::codegen::Env>::EnvAccess as ::core::default::Default>::default()
            }
        }
        impl<'a> ::ink::codegen::StaticEnv for QuickcheckTests {
            type EnvAccess = ::ink::EnvAccess<
                'static,
                <QuickcheckTests as ::ink::env::ContractEnv>::Env,
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
    impl ::ink::reflect::ContractAmountDispatchables for QuickcheckTests {
        const MESSAGES: ::core::primitive::usize = 2usize;
        const CONSTRUCTORS: ::core::primitive::usize = 2usize;
    }
    impl ::ink::reflect::ContractDispatchableMessages<
        { <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES },
    > for QuickcheckTests {
        const IDS: [::core::primitive::u32; <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES] = [
            0x1D32619F_u32,
            0x2F865BD9_u32,
        ];
    }
    impl ::ink::reflect::ContractDispatchableConstructors<
        {
            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
        },
    > for QuickcheckTests {
        const IDS: [::core::primitive::u32; <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS] = [
            0x9BAE9D5E_u32,
            0x61EF7E3E_u32,
        ];
    }
    impl ::ink::reflect::DispatchableConstructorInfo<0x9BAE9D5E_u32>
    for QuickcheckTests {
        type Input = i32;
        type Output = Self;
        type Storage = QuickcheckTests;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<QuickcheckTests>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<QuickcheckTests>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |__ink_binding_0| {
            QuickcheckTests::new(__ink_binding_0)
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
    impl ::ink::reflect::DispatchableConstructorInfo<0x61EF7E3E_u32>
    for QuickcheckTests {
        type Input = ();
        type Output = Self;
        type Storage = QuickcheckTests;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<QuickcheckTests>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<QuickcheckTests>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |_| {
            QuickcheckTests::new_default()
        };
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x61_u8,
            0xEF_u8,
            0x7E_u8,
            0x3E_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new_default";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x1D32619F_u32> for QuickcheckTests {
        type Input = i32;
        type Output = ();
        type Storage = QuickcheckTests;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { QuickcheckTests::inc(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x1D_u8,
            0x32_u8,
            0x61_u8,
            0x9F_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "inc";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x2F865BD9_u32> for QuickcheckTests {
        type Input = ();
        type Output = i32;
        type Storage = QuickcheckTests;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { QuickcheckTests::get(storage) };
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
                <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                            },
                        >>::IDS[0usize]
                    },
                >>::Input,
            ),
            Constructor1(
                <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
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
                const CONSTRUCTOR_0: [::core::primitive::u8; 4usize] = <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                            },
                        >>::IDS[0usize]
                    },
                >>::SELECTOR;
                const CONSTRUCTOR_1: [::core::primitive::u8; 4usize] = <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
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
                                <<QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
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
                                <<QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
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
                                || <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::PAYABLE
                                || <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::PAYABLE
                        }
                            && !<QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[0usize]
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <QuickcheckTests as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[0usize]
                            },
                        >>::Output = <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[0usize]
                            },
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[0usize]
                                },
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            QuickcheckTests,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                QuickcheckTests,
                            >(
                                &<QuickcheckTests as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                                    {
                                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                                    },
                                                >>::IDS[0usize]
                                            },
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<
                                        QuickcheckTests,
                                    >>::Error,
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
                                || <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::PAYABLE
                                || <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::PAYABLE
                        }
                            && !<QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[1usize]
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <QuickcheckTests as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[1usize]
                            },
                        >>::Output = <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                    },
                                >>::IDS[1usize]
                            },
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                        {
                                            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                        },
                                    >>::IDS[1usize]
                                },
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            QuickcheckTests,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                QuickcheckTests,
                            >(
                                &<QuickcheckTests as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                                    {
                                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                                    },
                                                >>::IDS[1usize]
                                            },
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<
                                        QuickcheckTests,
                                    >>::Error,
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
        impl ::ink::reflect::ContractConstructorDecoder for QuickcheckTests {
            type Type = __ink_ConstructorDecoder;
        }
    };
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_MessageDecoder {
            Message0(
                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                            },
                        >>::IDS[0usize]
                    },
                >>::Input,
            ),
            Message1(
                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                            },
                        >>::IDS[1usize]
                    },
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
                const MESSAGE_0: [::core::primitive::u8; 4usize] = <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                            },
                        >>::IDS[0usize]
                    },
                >>::SELECTOR;
                const MESSAGE_1: [::core::primitive::u8; 4usize] = <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                    {
                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                            },
                        >>::IDS[1usize]
                    },
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    MESSAGE_0 => {
                        ::core::result::Result::Ok(
                            Self::Message0(
                                <<QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
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
                    MESSAGE_1 => {
                        ::core::result::Result::Ok(
                            Self::Message1(
                                <<QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
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
        impl ::scale::Decode for __ink_MessageDecoder {
            fn decode<I>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error>
            where
                I: ::scale::Input,
            {
                <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                    .map_err(::core::convert::Into::into)
            }
        }
        fn push_contract(
            contract: ::core::mem::ManuallyDrop<QuickcheckTests>,
            mutates: bool,
        ) {
            if mutates {
                ::ink::env::set_contract_storage::<
                    ::ink::primitives::Key,
                    QuickcheckTests,
                >(
                    &<QuickcheckTests as ::ink::storage::traits::StorageKey>::KEY,
                    &contract,
                );
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_MessageDecoder {
            #[allow(clippy::nonminimal_bool, clippy::let_unit_value)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                let key = <QuickcheckTests as ::ink::storage::traits::StorageKey>::KEY;
                let mut contract: ::core::mem::ManuallyDrop<QuickcheckTests> = ::core::mem::ManuallyDrop::new(
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
                        if {
                            false
                                || <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::PAYABLE
                                || <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::PAYABLE
                        }
                            && !<QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                        {
                                            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                        },
                                    >>::IDS[0usize]
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <QuickcheckTests as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                    },
                                >>::IDS[0usize]
                            },
                        >>::Output = <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                    },
                                >>::IDS[0usize]
                            },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[0usize]
                                    },
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
                                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    Self::Message1(input) => {
                        if {
                            false
                                || <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[0usize]
                                    },
                                >>::PAYABLE
                                || <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::PAYABLE
                        }
                            && !<QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                        {
                                            <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                        },
                                    >>::IDS[1usize]
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <QuickcheckTests as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                    },
                                >>::IDS[1usize]
                            },
                        >>::Output = <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                            {
                                <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                    },
                                >>::IDS[1usize]
                            },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[1usize]
                                    },
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
                                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[1usize]
                                    },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                            {
                                                <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                            },
                                        >>::IDS[1usize]
                                    },
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
        impl ::ink::reflect::ContractMessageDecoder for QuickcheckTests {
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
                    || <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                        {
                            <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                },
                            >>::IDS[0usize]
                        },
                    >>::PAYABLE
                    || <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                        {
                            <QuickcheckTests as ::ink::reflect::ContractDispatchableConstructors<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                                },
                            >>::IDS[1usize]
                        },
                    >>::PAYABLE
            } {
                ::ink::codegen::deny_payment::<
                    <QuickcheckTests as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| ::core::panicking::panic_display(&error))
            }
            let dispatchable = match ::ink::env::decode_input::<
                <QuickcheckTests as ::ink::reflect::ContractConstructorDecoder>::Type,
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
            <<QuickcheckTests as ::ink::reflect::ContractConstructorDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
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
            if !{
                false
                    || <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                        {
                            <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                },
                            >>::IDS[0usize]
                        },
                    >>::PAYABLE
                    || <QuickcheckTests as ::ink::reflect::DispatchableMessageInfo<
                        {
                            <QuickcheckTests as ::ink::reflect::ContractDispatchableMessages<
                                {
                                    <QuickcheckTests as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                                },
                            >>::IDS[1usize]
                        },
                    >>::PAYABLE
            } {
                ::ink::codegen::deny_payment::<
                    <QuickcheckTests as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| ::core::panicking::panic_display(&error))
            }
            let dispatchable = match ::ink::env::decode_input::<
                <QuickcheckTests as ::ink::reflect::ContractMessageDecoder>::Type,
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
            <<QuickcheckTests as ::ink::reflect::ContractMessageDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
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
        const _: ::ink::codegen::utils::IsSameType<QuickcheckTests> = ::ink::codegen::utils::IsSameType::<
            QuickcheckTests,
        >::new();
        impl QuickcheckTests {
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new(init_value: i32) -> Self {
                Self { value: init_value }
            }
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new_default() -> Self {
                Self::new(Default::default())
            }
            pub fn inc(&mut self, by: i32) {
                self.value += by;
            }
            pub fn get(&self) -> i32 {
                self.value
            }
        }
        const _: () = {
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<i32>>();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<i32>>();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchOutput<i32>>();
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
                        .path(
                            ::scale_info::Path::new(
                                "CallBuilder",
                                "quickcheck_tests::quickcheck_tests",
                            ),
                        )
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
            impl ::ink::codegen::ContractCallBuilder for QuickcheckTests {
                type Type = CallBuilder;
            }
            impl ::ink::env::ContractEnv for CallBuilder {
                type Env = <QuickcheckTests as ::ink::env::ContractEnv>::Env;
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
        impl CallBuilder {
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn inc(
                &mut self,
                __ink_binding_0: i32,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<i32>,
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
                                    0x1D_u8,
                                    0x32_u8,
                                    0x61_u8,
                                    0x9F_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<()>()
            }
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
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<i32>>,
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
                    .returns::<i32>()
            }
        }
    };
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    pub struct QuickcheckTestsRef {
        inner: <QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for QuickcheckTestsRef {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "QuickcheckTestsRef",
                "inner",
                &&self.inner,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for QuickcheckTestsRef {
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
        impl ::scale::EncodeLike for QuickcheckTestsRef {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Decode for QuickcheckTestsRef {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(QuickcheckTestsRef {
                    inner: {
                        let __codec_res_edqy = <<QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `QuickcheckTestsRef::inner`"),
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
    impl ::core::hash::Hash for QuickcheckTestsRef {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.inner, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for QuickcheckTestsRef {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for QuickcheckTestsRef {
        #[inline]
        fn eq(&self, other: &QuickcheckTestsRef) -> bool {
            self.inner == other.inner
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for QuickcheckTestsRef {}
    #[automatically_derived]
    impl ::core::cmp::Eq for QuickcheckTestsRef {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<
                <QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type,
            >;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for QuickcheckTestsRef {
        #[inline]
        fn clone(&self) -> QuickcheckTestsRef {
            QuickcheckTestsRef {
                inner: ::core::clone::Clone::clone(&self.inner),
            }
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for QuickcheckTestsRef {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new(
                            "QuickcheckTestsRef",
                            "quickcheck_tests::quickcheck_tests",
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .docs(
                        &[
                            "Defines the storage of your contract.",
                            "Add new fields to the below struct in order",
                            "to add new static storage fields to your contract.",
                        ],
                    )
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type,
                                    >()
                                    .name("inner")
                                    .type_name(
                                        "<QuickcheckTests as::ink::codegen::ContractCallBuilder>::Type",
                                    )
                            }),
                    )
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for QuickcheckTestsRef {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "QuickcheckTestsRef",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "inner",
                                <<QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type as ::ink::storage::traits::StorageLayout>::layout(
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
        impl ::ink::env::ContractReference for QuickcheckTests {
            type Type = QuickcheckTestsRef;
        }
        impl ::ink::env::call::ConstructorReturnType<QuickcheckTestsRef>
        for QuickcheckTests {
            type Output = QuickcheckTestsRef;
            type Error = ();
            fn ok(value: QuickcheckTestsRef) -> Self::Output {
                value
            }
        }
        impl<E> ::ink::env::call::ConstructorReturnType<QuickcheckTestsRef>
        for ::core::result::Result<QuickcheckTests, E>
        where
            E: ::scale::Decode,
        {
            const IS_RESULT: bool = true;
            type Output = ::core::result::Result<QuickcheckTestsRef, E>;
            type Error = E;
            fn ok(value: QuickcheckTestsRef) -> Self::Output {
                ::core::result::Result::Ok(value)
            }
            fn err(err: Self::Error) -> ::core::option::Option<Self::Output> {
                ::core::option::Option::Some(::core::result::Result::Err(err))
            }
        }
        impl ::ink::env::ContractEnv for QuickcheckTestsRef {
            type Env = <QuickcheckTests as ::ink::env::ContractEnv>::Env;
        }
    };
    impl QuickcheckTestsRef {
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new(
            __ink_binding_0: i32,
        ) -> ::ink::env::call::CreateBuilder<
            Environment,
            Self,
            ::ink::env::call::utils::Unset<Hash>,
            ::ink::env::call::utils::Unset<u64>,
            ::ink::env::call::utils::Unset<Balance>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::ArgumentList<
                        ::ink::env::call::utils::Argument<i32>,
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
        #[inline]
        pub fn inc(&mut self, by: i32) {
            self.try_inc(by)
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "QuickcheckTests", "inc", error
                    ),
                ))
        }
        #[inline]
        pub fn try_inc(&mut self, by: i32) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .inc(by)
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "QuickcheckTests", "inc", error
                    ),
                ))
        }
        #[inline]
        pub fn get(&self) -> i32 {
            self.try_get()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "QuickcheckTests", "get", error
                    ),
                ))
        }
        #[inline]
        pub fn try_get(&self) -> ::ink::MessageResult<i32> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .get()
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "QuickcheckTests", "get", error
                    ),
                ))
        }
    }
    const _: () = {
        impl ::ink::codegen::TraitCallBuilder for QuickcheckTestsRef {
            type Builder = <QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type;
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
    impl ::ink::env::call::FromAccountId<Environment> for QuickcheckTestsRef {
        #[inline]
        fn from_account_id(account_id: AccountId) -> Self {
            Self {
                inner: <<QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type as ::ink::env::call::FromAccountId<
                    Environment,
                >>::from_account_id(account_id),
            }
        }
    }
    impl ::ink::ToAccountId<Environment> for QuickcheckTestsRef {
        #[inline]
        fn to_account_id(&self) -> AccountId {
            <<QuickcheckTests as ::ink::codegen::ContractCallBuilder>::Type as ::ink::ToAccountId<
                Environment,
            >>::to_account_id(&self.inner)
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
                    >>::from(
                        <QuickcheckTests as ::ink::storage::traits::StorageKey>::KEY,
                    ),
                    <QuickcheckTests as ::ink::storage::traits::StorageLayout>::layout(
                        &<QuickcheckTests as ::ink::storage::traits::StorageKey>::KEY,
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
                                            i32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["i32"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                        2611912030u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
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
                            .docs([])
                            .done(),
                        ::ink::metadata::ConstructorSpec::from_label("new_default")
                            .selector([0x61_u8, 0xEF_u8, 0x7E_u8, 0x3E_u8])
                            .args([])
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
                                        1643085374u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <QuickcheckTests as ::ink::reflect::DispatchableConstructorInfo<
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
                            .docs([])
                            .done(),
                    ])
                    .messages([
                        ::ink::metadata::MessageSpec::from_label("inc")
                            .selector([0x1D_u8, 0x32_u8, 0x61_u8, 0x9F_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("by")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            i32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["i32"]),
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
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("get")
                            .selector([0x2F_u8, 0x86_u8, 0x5B_u8, 0xD9_u8])
                            .args([])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<i32>,
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
                            .docs([])
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
