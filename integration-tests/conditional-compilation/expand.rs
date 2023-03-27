#![feature(prelude_import)]
#![allow(clippy::new_without_default)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub trait Flip: ::ink::env::ContractEnv {
    /// Holds general and global information about the trait.
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    type __ink_TraitInfo: ::ink::codegen::TraitCallForwarder;
    /// Output type of the respective trait message.
    type flipOutput: ::ink::codegen::ImpliesReturn<()>;
    /// Flips the current value of the Flipper's boolean.
    fn flip(&mut self) -> Self::flipOutput;
    /// Output type of the respective trait message.
    type getOutput: ::ink::codegen::ImpliesReturn<bool>;
    /// Returns the current value of the Flipper's boolean.
    fn get(&self) -> Self::getOutput;
    /// Output type of the respective trait message.
    #[cfg(feature = "foo")]
    type pushFooOutput: ::ink::codegen::ImpliesReturn<()>;
    #[cfg(feature = "foo")]
    fn push_foo(&mut self, value: bool) -> Self::pushFooOutput;
}
const _: () = {
    impl<E> Flip for ::ink::reflect::TraitDefinitionRegistry<E>
    where
        E: ::ink::env::Environment,
    {
        /// Holds general and global information about the trait.
        #[allow(non_camel_case_types)]
        type __ink_TraitInfo = __ink_TraitInfoFlip<E>;
        type flipOutput = ();
        /// Flips the current value of the Flipper's boolean.
        #[cold]
        fn flip(&mut self) -> Self::flipOutput {
            /// We enforce linking errors in case this is ever actually called.
            /// These linker errors are properly resolved by the cargo-contract tool.
            extern {
                fn __ink_enforce_error_0x0110466c697010666c6970aa97cade01() -> !;
            }
            unsafe { __ink_enforce_error_0x0110466c697010666c6970aa97cade01() }
        }
        type getOutput = bool;
        /// Returns the current value of the Flipper's boolean.
        #[cold]
        fn get(&self) -> Self::getOutput {
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<bool>,
            >();
            /// We enforce linking errors in case this is ever actually called.
            /// These linker errors are properly resolved by the cargo-contract tool.
            extern {
                fn __ink_enforce_error_0x0110466c69700c67657484693fb200() -> !;
            }
            unsafe { __ink_enforce_error_0x0110466c69700c67657484693fb200() }
        }
        #[cfg(feature = "foo")]
        type pushFooOutput = ();
        #[cfg(feature = "foo")]
        #[cold]
        fn push_foo(&mut self, __ink_binding_0: bool) -> Self::pushFooOutput {
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            /// We enforce linking errors in case this is ever actually called.
            /// These linker errors are properly resolved by the cargo-contract tool.
            extern {
                fn __ink_enforce_error_0x0110466c697020707573685f666f6f84417a2101() -> !;
            }
            unsafe { __ink_enforce_error_0x0110466c697020707573685f666f6f84417a2101() }
        }
    }
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub struct __ink_TraitInfoFlip<E> {
        marker: ::core::marker::PhantomData<fn() -> E>,
    }
    impl<E> ::ink::reflect::TraitMessageInfo<1664787793u32> for __ink_TraitInfoFlip<E> {
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xAA_u8,
            0x97_u8,
            0xCA_u8,
            0xDE_u8,
        ];
    }
    impl<E> ::ink::reflect::TraitMessageInfo<797334489u32> for __ink_TraitInfoFlip<E> {
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x84_u8,
            0x69_u8,
            0x3F_u8,
            0xB2_u8,
        ];
    }
    impl<E> ::ink::reflect::TraitMessageInfo<2182276109u32> for __ink_TraitInfoFlip<E> {
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x84_u8,
            0x41_u8,
            0x7A_u8,
            0x21_u8,
        ];
    }
    impl<E> ::ink::reflect::TraitInfo for __ink_TraitInfoFlip<E>
    where
        E: ::ink::env::Environment,
    {
        const ID: u32 = 2864680781;
        const PATH: &'static ::core::primitive::str = "conditional_compilation";
        const NAME: &'static ::core::primitive::str = "Flip";
    }
    impl<E> ::ink::codegen::TraitCallForwarder for __ink_TraitInfoFlip<E>
    where
        E: ::ink::env::Environment,
    {
        type Forwarder = __ink_TraitCallForwarderFlip<E>;
    }
    /// The global call builder type for all trait implementers.
    ///
    /// All calls to types (contracts) implementing the trait will be built by this type.
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    #[repr(transparent)]
    pub struct __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        account_id: <E as ::ink::env::Environment>::AccountId,
    }
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl<E> ::scale::Encode for __ink_TraitCallBuilderFlip<E>
        where
            E: ::ink::env::Environment,
            <E as ::ink::env::Environment>::AccountId: ::scale::Encode,
            <E as ::ink::env::Environment>::AccountId: ::scale::Encode,
        {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::scale::Encode::encode_to(&&self.account_id, __codec_dest_edqy)
            }
            fn encode(&self) -> ::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::scale::Encode::encode(&&self.account_id)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::scale::Encode::using_encoded(&&self.account_id, f)
            }
        }
        #[automatically_derived]
        impl<E> ::scale::EncodeLike for __ink_TraitCallBuilderFlip<E>
        where
            E: ::ink::env::Environment,
            <E as ::ink::env::Environment>::AccountId: ::scale::Encode,
            <E as ::ink::env::Environment>::AccountId: ::scale::Encode,
        {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl<E> ::scale::Decode for __ink_TraitCallBuilderFlip<E>
        where
            E: ::ink::env::Environment,
            <E as ::ink::env::Environment>::AccountId: ::scale::Decode,
            <E as ::ink::env::Environment>::AccountId: ::scale::Decode,
        {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(__ink_TraitCallBuilderFlip::<E> {
                    account_id: {
                        let __codec_res_edqy = <<E as ::ink::env::Environment>::AccountId as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e
                                        .chain(
                                            "Could not decode `__ink_TraitCallBuilderFlip::account_id`",
                                        ),
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
    #[cfg(feature = "std")]
    impl<E> ::ink::storage::traits::StorageLayout for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::ink::storage::traits::StorageLayout,
    {
        fn layout(__key: &::ink::primitives::Key) -> ::ink::metadata::layout::Layout {
            ::ink::metadata::layout::Layout::Struct(
                ::ink::metadata::layout::StructLayout::new(
                    "__ink_TraitCallBuilderFlip",
                    [
                        ::ink::metadata::layout::FieldLayout::new(
                            "account_id",
                            <<E as ::ink::env::Environment>::AccountId as ::ink::storage::traits::StorageLayout>::layout(
                                __key,
                            ),
                        ),
                    ],
                ),
            )
        }
    }
    /// We require this manual implementation since the derive produces incorrect trait bounds.
    impl<E> ::core::clone::Clone for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::core::clone::Clone,
    {
        #[inline]
        fn clone(&self) -> Self {
            Self {
                account_id: ::core::clone::Clone::clone(&self.account_id),
            }
        }
    }
    /// We require this manual implementation since the derive produces incorrect trait bounds.
    impl<E> ::core::fmt::Debug for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            f.debug_struct("__ink_TraitCallBuilderFlip")
                .field("account_id", &self.account_id)
                .finish()
        }
    }
    #[cfg(feature = "std")]
    /// We require this manual implementation since the derive produces incorrect trait bounds.
    impl<E> ::scale_info::TypeInfo for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::scale_info::TypeInfo + 'static,
    {
        type Identity = <E as ::ink::env::Environment>::AccountId;
        fn type_info() -> ::scale_info::Type {
            <<E as ::ink::env::Environment>::AccountId as ::scale_info::TypeInfo>::type_info()
        }
    }
    impl<E> ::ink::env::call::FromAccountId<E> for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        #[inline]
        fn from_account_id(
            account_id: <E as ::ink::env::Environment>::AccountId,
        ) -> Self {
            Self { account_id }
        }
    }
    impl<E, AccountId> ::core::convert::From<AccountId> for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment<AccountId = AccountId>,
        AccountId: ::ink::env::AccountIdGuard,
    {
        fn from(value: AccountId) -> Self {
            <Self as ::ink::env::call::FromAccountId<E>>::from_account_id(value)
        }
    }
    impl<E> ::ink::ToAccountId<E> for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        #[inline]
        fn to_account_id(&self) -> <E as ::ink::env::Environment>::AccountId {
            <<E as ::ink::env::Environment>::AccountId as ::core::clone::Clone>::clone(
                &self.account_id,
            )
        }
    }
    impl<E, AccountId> ::core::convert::AsRef<AccountId>
    for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment<AccountId = AccountId>,
    {
        fn as_ref(&self) -> &AccountId {
            &self.account_id
        }
    }
    impl<E, AccountId> ::core::convert::AsMut<AccountId>
    for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment<AccountId = AccountId>,
    {
        fn as_mut(&mut self) -> &mut AccountId {
            &mut self.account_id
        }
    }
    impl<E> ::ink::env::ContractEnv for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        type Env = E;
    }
    impl<E> Flip for __ink_TraitCallBuilderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        #[allow(non_camel_case_types)]
        type __ink_TraitInfo = __ink_TraitInfoFlip<E>;
        #[allow(clippy::type_complexity)]
        type flipOutput = ::ink::env::call::CallBuilder<
            Self::Env,
            ::ink::env::call::utils::Set<::ink::env::call::Call<Self::Env>>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::EmptyArgumentList,
                >,
            >,
            ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
        >;
        /// Flips the current value of the Flipper's boolean.
        #[inline]
        fn flip(&mut self) -> Self::flipOutput {
            ::ink::env::call::build_call::<Self::Env>()
                .call(::ink::ToAccountId::to_account_id(self))
                .exec_input(
                    ::ink::env::call::ExecutionInput::new(
                        ::ink::env::call::Selector::new([
                            0xAA_u8,
                            0x97_u8,
                            0xCA_u8,
                            0xDE_u8,
                        ]),
                    ),
                )
                .returns::<()>()
        }
        #[allow(clippy::type_complexity)]
        type getOutput = ::ink::env::call::CallBuilder<
            Self::Env,
            ::ink::env::call::utils::Set<::ink::env::call::Call<Self::Env>>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::EmptyArgumentList,
                >,
            >,
            ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<bool>>,
        >;
        /// Returns the current value of the Flipper's boolean.
        #[inline]
        fn get(&self) -> Self::getOutput {
            ::ink::env::call::build_call::<Self::Env>()
                .call(::ink::ToAccountId::to_account_id(self))
                .exec_input(
                    ::ink::env::call::ExecutionInput::new(
                        ::ink::env::call::Selector::new([
                            0x84_u8,
                            0x69_u8,
                            0x3F_u8,
                            0xB2_u8,
                        ]),
                    ),
                )
                .returns::<bool>()
        }
        #[allow(clippy::type_complexity)]
        #[cfg(feature = "foo")]
        type pushFooOutput = ::ink::env::call::CallBuilder<
            Self::Env,
            ::ink::env::call::utils::Set<::ink::env::call::Call<Self::Env>>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::ArgumentList<
                        ::ink::env::call::utils::Argument<bool>,
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
            >,
            ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
        >;
        #[cfg(feature = "foo")]
        #[inline]
        fn push_foo(&mut self, __ink_binding_0: bool) -> Self::pushFooOutput {
            ::ink::env::call::build_call::<Self::Env>()
                .call(::ink::ToAccountId::to_account_id(self))
                .exec_input(
                    ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0x84_u8,
                                0x41_u8,
                                0x7A_u8,
                                0x21_u8,
                            ]),
                        )
                        .push_arg(__ink_binding_0),
                )
                .returns::<()>()
        }
    }
    /// The global call forwarder for the ink! trait definition.
    ///
    /// All cross-contract calls to contracts implementing the associated ink! trait
    /// will be handled by this type.
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    #[repr(transparent)]
    pub struct __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        builder: <Self as ::ink::codegen::TraitCallBuilder>::Builder,
    }
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl<E> ::scale::Encode for __ink_TraitCallForwarderFlip<E>
        where
            E: ::ink::env::Environment,
        {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::scale::Encode::encode_to(&&self.builder, __codec_dest_edqy)
            }
            fn encode(&self) -> ::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::scale::Encode::encode(&&self.builder)
            }
            fn using_encoded<R, F: ::core::ops::FnOnce(&[::core::primitive::u8]) -> R>(
                &self,
                f: F,
            ) -> R {
                ::scale::Encode::using_encoded(&&self.builder, f)
            }
        }
        #[automatically_derived]
        impl<E> ::scale::EncodeLike for __ink_TraitCallForwarderFlip<E>
        where
            E: ::ink::env::Environment,
        {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl<E> ::scale::Decode for __ink_TraitCallForwarderFlip<E>
        where
            E: ::ink::env::Environment,
        {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(__ink_TraitCallForwarderFlip::<E> {
                    builder: {
                        let __codec_res_edqy = <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e
                                        .chain(
                                            "Could not decode `__ink_TraitCallForwarderFlip::builder`",
                                        ),
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
    #[cfg(feature = "std")]
    impl<E> ::ink::storage::traits::StorageLayout for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::ink::storage::traits::StorageLayout,
    {
        fn layout(__key: &::ink::primitives::Key) -> ::ink::metadata::layout::Layout {
            <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::ink::storage::traits::StorageLayout>::layout(
                __key,
            )
        }
    }
    /// We require this manual implementation since the derive produces incorrect trait bounds.
    impl<E> ::core::clone::Clone for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::core::clone::Clone,
    {
        #[inline]
        fn clone(&self) -> Self {
            Self {
                builder: <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::core::clone::Clone>::clone(
                    &self.builder,
                ),
            }
        }
    }
    /// We require this manual implementation since the derive produces incorrect trait bounds.
    impl<E> ::core::fmt::Debug for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::core::fmt::Debug,
    {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            f.debug_struct("__ink_TraitCallForwarderFlip")
                .field("account_id", &self.builder.account_id)
                .finish()
        }
    }
    #[cfg(feature = "std")]
    /// We require this manual implementation since the derive produces incorrect trait bounds.
    impl<E> ::scale_info::TypeInfo for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
        <E as ::ink::env::Environment>::AccountId: ::scale_info::TypeInfo + 'static,
    {
        type Identity = <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::scale_info::TypeInfo>::Identity;
        fn type_info() -> ::scale_info::Type {
            <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::scale_info::TypeInfo>::type_info()
        }
    }
    impl<E> ::ink::env::call::FromAccountId<E> for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        #[inline]
        fn from_account_id(
            account_id: <E as ::ink::env::Environment>::AccountId,
        ) -> Self {
            Self {
                builder: <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::ink::env::call::FromAccountId<
                    E,
                >>::from_account_id(account_id),
            }
        }
    }
    impl<E, AccountId> ::core::convert::From<AccountId>
    for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment<AccountId = AccountId>,
        AccountId: ::ink::env::AccountIdGuard,
    {
        fn from(value: AccountId) -> Self {
            <Self as ::ink::env::call::FromAccountId<E>>::from_account_id(value)
        }
    }
    impl<E> ::ink::ToAccountId<E> for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        #[inline]
        fn to_account_id(&self) -> <E as ::ink::env::Environment>::AccountId {
            <<Self as ::ink::codegen::TraitCallBuilder>::Builder as ::ink::ToAccountId<
                E,
            >>::to_account_id(&self.builder)
        }
    }
    impl<E, AccountId> ::core::convert::AsRef<AccountId>
    for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment<AccountId = AccountId>,
    {
        fn as_ref(&self) -> &AccountId {
            <_ as ::core::convert::AsRef<AccountId>>::as_ref(&self.builder)
        }
    }
    impl<E, AccountId> ::core::convert::AsMut<AccountId>
    for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment<AccountId = AccountId>,
    {
        fn as_mut(&mut self) -> &mut AccountId {
            <_ as ::core::convert::AsMut<AccountId>>::as_mut(&mut self.builder)
        }
    }
    /// This trait allows to bridge from call forwarder to call builder.
    ///
    /// Also this explains why we designed the generated code so that we have
    /// both types and why the forwarder is a thin-wrapper around the builder
    /// as this allows to perform this operation safely.
    impl<E> ::ink::codegen::TraitCallBuilder for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        type Builder = __ink_TraitCallBuilderFlip<E>;
        #[inline]
        fn call(&self) -> &<Self as ::ink::codegen::TraitCallBuilder>::Builder {
            &self.builder
        }
        #[inline]
        fn call_mut(
            &mut self,
        ) -> &mut <Self as ::ink::codegen::TraitCallBuilder>::Builder {
            &mut self.builder
        }
    }
    impl<E> ::ink::env::ContractEnv for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        type Env = E;
    }
    impl<E> Flip for __ink_TraitCallForwarderFlip<E>
    where
        E: ::ink::env::Environment,
    {
        #[allow(non_camel_case_types)]
        type __ink_TraitInfo = __ink_TraitInfoFlip<E>;
        type flipOutput = ();
        /// Flips the current value of the Flipper's boolean.
        #[inline]
        fn flip(&mut self) -> Self::flipOutput {
            <<Self as ::ink::codegen::TraitCallBuilder>::Builder as Flip>::flip(
                    <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self),
                )
                .try_invoke()
                .unwrap_or_else(|env_err| ::core::panicking::panic_fmt(
                    format_args!(
                        "{0}: {1:?}",
                        "encountered error while calling <__ink_TraitCallForwarderFlip as Flip>::flip",
                        env_err
                    ),
                ))
                .unwrap_or_else(|lang_err| ::core::panicking::panic_fmt(
                    format_args!(
                        "{0}: {1:?}",
                        "encountered error while calling <__ink_TraitCallForwarderFlip as Flip>::flip",
                        lang_err
                    ),
                ))
        }
        type getOutput = bool;
        /// Returns the current value of the Flipper's boolean.
        #[inline]
        fn get(&self) -> Self::getOutput {
            <<Self as ::ink::codegen::TraitCallBuilder>::Builder as Flip>::get(
                    <Self as ::ink::codegen::TraitCallBuilder>::call(self),
                )
                .try_invoke()
                .unwrap_or_else(|env_err| ::core::panicking::panic_fmt(
                    format_args!(
                        "{0}: {1:?}",
                        "encountered error while calling <__ink_TraitCallForwarderFlip as Flip>::get",
                        env_err
                    ),
                ))
                .unwrap_or_else(|lang_err| ::core::panicking::panic_fmt(
                    format_args!(
                        "{0}: {1:?}",
                        "encountered error while calling <__ink_TraitCallForwarderFlip as Flip>::get",
                        lang_err
                    ),
                ))
        }
        #[cfg(feature = "foo")]
        type pushFooOutput = ();
        #[cfg(feature = "foo")]
        #[inline]
        fn push_foo(&mut self, value: bool) -> Self::pushFooOutput {
            <<Self as ::ink::codegen::TraitCallBuilder>::Builder as Flip>::push_foo(
                    <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self),
                    value,
                )
                .try_invoke()
                .unwrap_or_else(|env_err| ::core::panicking::panic_fmt(
                    format_args!(
                        "{0}: {1:?}",
                        "encountered error while calling <__ink_TraitCallForwarderFlip as Flip>::push_foo",
                        env_err
                    ),
                ))
                .unwrap_or_else(|lang_err| ::core::panicking::panic_fmt(
                    format_args!(
                        "{0}: {1:?}",
                        "encountered error while calling <__ink_TraitCallForwarderFlip as Flip>::push_foo",
                        lang_err
                    ),
                ))
        }
    }
};
pub mod conditional_compilation {
    impl ::ink::env::ContractEnv for ConditionalCompilation {
        type Env = ::ink::env::DefaultEnvironment;
    }
    type Environment = <ConditionalCompilation as ::ink::env::ContractEnv>::Env;
    type AccountId = <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::AccountId;
    type Balance = <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Balance;
    type Hash = <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Hash;
    type Timestamp = <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Timestamp;
    type BlockNumber = <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::BlockNumber;
    const _: () = {
        struct Check {
            salt: (),
            field_0: bool,
        }
    };
    #[cfg(not(feature = "__ink_dylint_Storage"))]
    pub struct ConditionalCompilation {
        value: <bool as ::ink::storage::traits::AutoStorableHint<
            ::ink::storage::traits::ManualKey<4189525415u32, ()>,
        >>::Type,
    }
    const _: () = {
        impl<
            __ink_generic_salt: ::ink::storage::traits::StorageKey,
        > ::ink::storage::traits::StorableHint<__ink_generic_salt>
        for ConditionalCompilation {
            type Type = ConditionalCompilation;
            type PreferredKey = ::ink::storage::traits::AutoKey;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageKey for ConditionalCompilation {
            const KEY: ::ink::primitives::Key = <() as ::ink::storage::traits::StorageKey>::KEY;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::Storable for ConditionalCompilation {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::scale::Input>(
                __input: &mut __ink_I,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(ConditionalCompilation {
                    value: <<bool as ::ink::storage::traits::AutoStorableHint<
                        ::ink::storage::traits::ManualKey<4189525415u32, ()>,
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
                    ConditionalCompilation { value: __binding_0 } => {
                        ::ink::storage::traits::Storable::encode(__binding_0, __dest);
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for ConditionalCompilation {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new(
                            "ConditionalCompilation",
                            "conditional_compilation::conditional_compilation",
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <bool as ::ink::storage::traits::AutoStorableHint<
                                            ::ink::storage::traits::ManualKey<4189525415u32, ()>,
                                        >>::Type,
                                    >()
                                    .name("value")
                                    .type_name(
                                        "<bool as::ink::storage::traits::AutoStorableHint<::ink::storage\n::traits::ManualKey<4189525415u32, ()>,>>::Type",
                                    )
                            }),
                    )
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for ConditionalCompilation {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "ConditionalCompilation",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "value",
                                <<bool as ::ink::storage::traits::AutoStorableHint<
                                    ::ink::storage::traits::ManualKey<4189525415u32, ()>,
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
        impl ::ink::reflect::ContractName for ConditionalCompilation {
            const NAME: &'static str = "ConditionalCompilation";
        }
    };
    const _: () = {
        impl<'a> ::ink::codegen::Env for &'a ConditionalCompilation {
            type EnvAccess = ::ink::EnvAccess<
                'a,
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            >;
            fn env(self) -> Self::EnvAccess {
                <<Self as ::ink::codegen::Env>::EnvAccess as ::core::default::Default>::default()
            }
        }
        impl<'a> ::ink::codegen::StaticEnv for ConditionalCompilation {
            type EnvAccess = ::ink::EnvAccess<
                'static,
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            >;
            fn env() -> Self::EnvAccess {
                <<Self as ::ink::codegen::StaticEnv>::EnvAccess as ::core::default::Default>::default()
            }
        }
    };
    const _: () = {
        #[allow(unused_imports)]
        use ::ink::codegen::{Env as _, StaticEnv as _};
        use ::ink::codegen::EmitEvent as _;
    };
    const _: () = {
        impl<'a> ::ink::codegen::EmitEvent<ConditionalCompilation>
        for ::ink::EnvAccess<'a, Environment> {
            fn emit_event<E>(self, event: E)
            where
                E: Into<
                    <ConditionalCompilation as ::ink::reflect::ContractEventBase>::Type,
                >,
            {
                ::ink::env::emit_event::<
                    Environment,
                    <ConditionalCompilation as ::ink::reflect::ContractEventBase>::Type,
                >(event.into());
            }
        }
    };
    #[allow(non_camel_case_types)]
    #[cfg(not(feature = "__ink_dylint_EventBase"))]
    pub enum __ink_EventBase {
        #[cfg(feature = "foo")]
        Changes(Changes),
        #[cfg(feature = "bar")]
        ChangesDated(ChangesDated),
    }
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl ::scale::Encode for __ink_EventBase {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    __ink_EventBase::Changes(ref aa) => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    __ink_EventBase::ChangesDated(ref aa) => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::scale::EncodeLike for __ink_EventBase {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl ::scale::Decode for __ink_EventBase {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e
                            .chain(
                                "Could not decode `__ink_EventBase`, failed to read variant byte",
                            )
                    })?
                {
                    __codec_x_edqy if __codec_x_edqy
                        == 0usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(
                            __ink_EventBase::Changes({
                                let __codec_res_edqy = <Changes as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `__ink_EventBase::Changes.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    }
                    __codec_x_edqy if __codec_x_edqy
                        == 1usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(
                            __ink_EventBase::ChangesDated({
                                let __codec_res_edqy = <ChangesDated as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e
                                                .chain("Could not decode `__ink_EventBase::ChangesDated.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    }
                    _ => {
                        ::core::result::Result::Err(
                            <_ as ::core::convert::Into<
                                _,
                            >>::into(
                                "Could not decode `__ink_EventBase`, variant doesn't exist",
                            ),
                        )
                    }
                }
            }
        }
    };
    const _: () = {
        impl ::ink::reflect::ContractEventBase for ConditionalCompilation {
            type Type = __ink_EventBase;
        }
    };
    #[cfg(feature = "foo")]
    const _: () = {
        impl From<Changes> for __ink_EventBase {
            fn from(event: Changes) -> Self {
                Self::Changes(event)
            }
        }
    };
    #[cfg(feature = "bar")]
    const _: () = {
        impl From<ChangesDated> for __ink_EventBase {
            fn from(event: ChangesDated) -> Self {
                Self::ChangesDated(event)
            }
        }
    };
    const _: () = {
        pub enum __ink_UndefinedAmountOfTopics {}
        impl ::ink::env::topics::EventTopicsAmount for __ink_UndefinedAmountOfTopics {
            const AMOUNT: usize = 0;
        }
        impl ::ink::env::Topics for __ink_EventBase {
            type RemainingTopics = __ink_UndefinedAmountOfTopics;
            fn topics<E, B>(
                &self,
                builder: ::ink::env::topics::TopicsBuilder<
                    ::ink::env::topics::state::Uninit,
                    E,
                    B,
                >,
            ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::topics::TopicsBuilderBackend<E>,
            {
                match self {
                    #[cfg(feature = "foo")]
                    Self::Changes(event) => {
                        <Changes as ::ink::env::Topics>::topics::<E, B>(event, builder)
                    }
                    #[cfg(feature = "bar")]
                    Self::ChangesDated(event) => {
                        <ChangesDated as ::ink::env::Topics>::topics::<
                            E,
                            B,
                        >(event, builder)
                    }
                    _ => {
                        ::core::panicking::panic_fmt(
                            format_args!("Event does not exist!"),
                        )
                    }
                }
            }
        }
    };
    #[cfg(feature = "foo")]
    impl ::ink::codegen::EventLenTopics for Changes {
        type LenTopics = ::ink::codegen::EventTopics<1usize>;
    }
    #[cfg(feature = "foo")]
    const _: () = ::ink::codegen::utils::consume_type::<
        ::ink::codegen::EventRespectsTopicLimit<
            Changes,
            {
                <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::MAX_EVENT_TOPICS
            },
        >,
    >();
    #[cfg(feature = "bar")]
    impl ::ink::codegen::EventLenTopics for ChangesDated {
        type LenTopics = ::ink::codegen::EventTopics<1usize>;
    }
    #[cfg(feature = "bar")]
    const _: () = ::ink::codegen::utils::consume_type::<
        ::ink::codegen::EventRespectsTopicLimit<
            ChangesDated,
            {
                <<ConditionalCompilation as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::MAX_EVENT_TOPICS
            },
        >,
    >();
    /// Feature gated event
    #[cfg(feature = "foo")]
    pub struct Changes {
        new_value: bool,
        by: AccountId,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for Changes {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::scale::Encode::encode_to(&self.new_value, __codec_dest_edqy);
                ::scale::Encode::encode_to(&self.by, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::scale::EncodeLike for Changes {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Decode for Changes {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(Changes {
                    new_value: {
                        let __codec_res_edqy = <bool as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Changes::new_value`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    by: {
                        let __codec_res_edqy = <AccountId as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `Changes::by`"),
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
    /// Feature gated event
    #[cfg(feature = "bar")]
    pub struct ChangesDated {
        new_value: bool,
        by: AccountId,
        when: BlockNumber,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for ChangesDated {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                ::scale::Encode::encode_to(&self.new_value, __codec_dest_edqy);
                ::scale::Encode::encode_to(&self.by, __codec_dest_edqy);
                ::scale::Encode::encode_to(&self.when, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::scale::EncodeLike for ChangesDated {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Decode for ChangesDated {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(ChangesDated {
                    new_value: {
                        let __codec_res_edqy = <bool as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `ChangesDated::new_value`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    by: {
                        let __codec_res_edqy = <AccountId as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `ChangesDated::by`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    when: {
                        let __codec_res_edqy = <BlockNumber as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `ChangesDated::when`"),
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
    #[cfg(feature = "foo")]
    const _: () = {
        impl ::ink::env::Topics for Changes {
            type RemainingTopics = [::ink::env::topics::state::HasRemainingTopics; 2usize];
            fn topics<E, B>(
                &self,
                builder: ::ink::env::topics::TopicsBuilder<
                    ::ink::env::topics::state::Uninit,
                    E,
                    B,
                >,
            ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::topics::TopicsBuilderBackend<E>,
            {
                builder
                    .build::<Self>()
                    .push_topic::<
                        ::ink::env::topics::PrefixedValue<[u8; 31usize]>,
                    >(
                        &::ink::env::topics::PrefixedValue {
                            value: b"ConditionalCompilation::Changes",
                            prefix: b"",
                        },
                    )
                    .push_topic::<
                        ::ink::env::topics::PrefixedValue<AccountId>,
                    >(
                        &::ink::env::topics::PrefixedValue {
                            value: &self.by,
                            prefix: b"ConditionalCompilation::Changes::by",
                        },
                    )
                    .finish()
            }
        }
    };
    #[cfg(feature = "bar")]
    const _: () = {
        impl ::ink::env::Topics for ChangesDated {
            type RemainingTopics = [::ink::env::topics::state::HasRemainingTopics; 2usize];
            fn topics<E, B>(
                &self,
                builder: ::ink::env::topics::TopicsBuilder<
                    ::ink::env::topics::state::Uninit,
                    E,
                    B,
                >,
            ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::topics::TopicsBuilderBackend<E>,
            {
                builder
                    .build::<Self>()
                    .push_topic::<
                        ::ink::env::topics::PrefixedValue<[u8; 36usize]>,
                    >(
                        &::ink::env::topics::PrefixedValue {
                            value: b"ConditionalCompilation::ChangesDated",
                            prefix: b"",
                        },
                    )
                    .push_topic::<
                        ::ink::env::topics::PrefixedValue<AccountId>,
                    >(
                        &::ink::env::topics::PrefixedValue {
                            value: &self.by,
                            prefix: b"ConditionalCompilation::ChangesDated::by",
                        },
                    )
                    .finish()
            }
        }
    };
    impl ::ink::reflect::DispatchableConstructorInfo<0x9BAE9D5E_u32>
    for ConditionalCompilation {
        type Input = ();
        type Output = Self;
        type Storage = ConditionalCompilation;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |_| {
            ConditionalCompilation::new()
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
    #[cfg(feature = "foo")]
    impl ::ink::reflect::DispatchableConstructorInfo<0xD362E73A_u32>
    for ConditionalCompilation {
        type Input = bool;
        type Output = Self;
        type Storage = ConditionalCompilation;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |__ink_binding_0| {
            ConditionalCompilation::new_foo(__ink_binding_0)
        };
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xD3_u8,
            0x62_u8,
            0xE7_u8,
            0x3A_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new_foo";
    }
    #[cfg(feature = "bar")]
    impl ::ink::reflect::DispatchableConstructorInfo<0x92670C4C_u32>
    for ConditionalCompilation {
        type Input = bool;
        type Output = Self;
        type Storage = ConditionalCompilation;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |__ink_binding_0| {
            ConditionalCompilation::new_bar(__ink_binding_0)
        };
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x92_u8,
            0x67_u8,
            0x0C_u8,
            0x4C_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new_bar";
    }
    #[cfg(feature = "foo")]
    #[cfg(feature = "bar")]
    impl ::ink::reflect::DispatchableConstructorInfo<0x8B85927D_u32>
    for ConditionalCompilation {
        type Input = bool;
        type Output = Self;
        type Storage = ConditionalCompilation;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<ConditionalCompilation>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |__ink_binding_0| {
            ConditionalCompilation::new_foo_bar(__ink_binding_0)
        };
        const PAYABLE: ::core::primitive::bool = false;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x8B_u8,
            0x85_u8,
            0x92_u8,
            0x7D_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new_foo_bar";
    }
    #[cfg(feature = "foo")]
    impl ::ink::reflect::DispatchableMessageInfo<0xC37844F9_u32>
    for ConditionalCompilation {
        type Input = ();
        type Output = ();
        type Storage = ConditionalCompilation;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { ConditionalCompilation::inherent_flip_foo(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xC3_u8,
            0x78_u8,
            0x44_u8,
            0xF9_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "inherent_flip_foo";
    }
    #[cfg(feature = "bar")]
    impl ::ink::reflect::DispatchableMessageInfo<0x9F45AF95_u32>
    for ConditionalCompilation {
        type Input = ();
        type Output = ();
        type Storage = ConditionalCompilation;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { ConditionalCompilation::inherent_flip_bar(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x9F_u8,
            0x45_u8,
            0xAF_u8,
            0x95_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "inherent_flip_bar";
    }
    impl ::ink::reflect::DispatchableMessageInfo<
        {
            ::core::primitive::u32::from_be_bytes({
                <<::ink::reflect::TraitDefinitionRegistry<
                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                    0x633AA551_u32,
                >>::SELECTOR
            })
        },
    > for ConditionalCompilation {
        type Input = ();
        type Output = ();
        type Storage = ConditionalCompilation;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { <ConditionalCompilation as Flip>::flip(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x633AA551_u32,
            >>::SELECTOR
        };
        const PAYABLE: ::core::primitive::bool = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x633AA551_u32,
            >>::PAYABLE
        };
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "Flip::flip";
    }
    impl ::ink::reflect::DispatchableMessageInfo<
        {
            ::core::primitive::u32::from_be_bytes({
                <<::ink::reflect::TraitDefinitionRegistry<
                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                    0x2F865BD9_u32,
                >>::SELECTOR
            })
        },
    > for ConditionalCompilation {
        type Input = ();
        type Output = bool;
        type Storage = ConditionalCompilation;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { <ConditionalCompilation as Flip>::get(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x2F865BD9_u32,
            >>::SELECTOR
        };
        const PAYABLE: ::core::primitive::bool = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x2F865BD9_u32,
            >>::PAYABLE
        };
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "Flip::get";
    }
    #[cfg(feature = "foo")]
    impl ::ink::reflect::DispatchableMessageInfo<
        {
            ::core::primitive::u32::from_be_bytes({
                <<::ink::reflect::TraitDefinitionRegistry<
                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                    0x8212E40D_u32,
                >>::SELECTOR
            })
        },
    > for ConditionalCompilation {
        type Input = bool;
        type Output = ();
        type Storage = ConditionalCompilation;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { <ConditionalCompilation as Flip>::push_foo(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x8212E40D_u32,
            >>::SELECTOR
        };
        const PAYABLE: ::core::primitive::bool = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x8212E40D_u32,
            >>::PAYABLE
        };
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "Flip::push_foo";
    }
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_ConstructorDecoder {
            Constructor0(
                <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0x9BAE9D5E_u32,
                >>::Input,
            ),
            #[cfg(feature = "foo")]
            Constructor1(
                <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0xD362E73A_u32,
                >>::Input,
            ),
            #[cfg(feature = "bar")]
            Constructor2(
                <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0x92670C4C_u32,
                >>::Input,
            ),
            #[cfg(feature = "foo")]
            #[cfg(feature = "bar")]
            Constructor3(
                <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0x8B85927D_u32,
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
                const CONSTRUCTOR_0: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0x9BAE9D5E_u32,
                >>::SELECTOR;
                #[cfg(feature = "foo")]
                const CONSTRUCTOR_1: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0xD362E73A_u32,
                >>::SELECTOR;
                #[cfg(feature = "bar")]
                const CONSTRUCTOR_2: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0x92670C4C_u32,
                >>::SELECTOR;
                #[cfg(feature = "foo")]
                #[cfg(feature = "bar")]
                const CONSTRUCTOR_3: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                    0x8B85927D_u32,
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    CONSTRUCTOR_0 => {
                        ::core::result::Result::Ok(
                            Self::Constructor0(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                    0x9BAE9D5E_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    #[cfg(feature = "foo")]
                    CONSTRUCTOR_1 => {
                        ::core::result::Result::Ok(
                            Self::Constructor1(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                    0xD362E73A_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    #[cfg(feature = "bar")]
                    CONSTRUCTOR_2 => {
                        ::core::result::Result::Ok(
                            Self::Constructor2(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                    0x92670C4C_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    #[cfg(feature = "foo")]
                    #[cfg(feature = "bar")]
                    CONSTRUCTOR_3 => {
                        ::core::result::Result::Ok(
                            Self::Constructor3(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                    0x8B85927D_u32,
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
                                || {
                                    let constructor_0 = false;
                                    let constructor_0 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x9BAE9D5E_u32,
                                    >>::PAYABLE;
                                    constructor_0
                                }
                                || {
                                    let constructor_1 = false;
                                    #[cfg(feature = "foo")]
                                    let constructor_1 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0xD362E73A_u32,
                                    >>::PAYABLE;
                                    constructor_1
                                }
                                || {
                                    let constructor_2 = false;
                                    #[cfg(feature = "bar")]
                                    let constructor_2 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x92670C4C_u32,
                                    >>::PAYABLE;
                                    constructor_2
                                }
                                || {
                                    let constructor_3 = false;
                                    #[cfg(feature = "foo")]
                                    #[cfg(feature = "bar")]
                                    let constructor_3 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x8B85927D_u32,
                                    >>::PAYABLE;
                                    constructor_3
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0x9BAE9D5E_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0x9BAE9D5E_u32,
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            ConditionalCompilation,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                ConditionalCompilation,
                            >(
                                &<ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                            0x9BAE9D5E_u32,
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<
                                        ConditionalCompilation,
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
                    #[cfg(feature = "foo")]
                    Self::Constructor1(input) => {
                        if {
                            false
                                || {
                                    let constructor_0 = false;
                                    let constructor_0 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x9BAE9D5E_u32,
                                    >>::PAYABLE;
                                    constructor_0
                                }
                                || {
                                    let constructor_1 = false;
                                    #[cfg(feature = "foo")]
                                    let constructor_1 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0xD362E73A_u32,
                                    >>::PAYABLE;
                                    constructor_1
                                }
                                || {
                                    let constructor_2 = false;
                                    #[cfg(feature = "bar")]
                                    let constructor_2 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x92670C4C_u32,
                                    >>::PAYABLE;
                                    constructor_2
                                }
                                || {
                                    let constructor_3 = false;
                                    #[cfg(feature = "foo")]
                                    #[cfg(feature = "bar")]
                                    let constructor_3 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x8B85927D_u32,
                                    >>::PAYABLE;
                                    constructor_3
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0xD362E73A_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0xD362E73A_u32,
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0xD362E73A_u32,
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0xD362E73A_u32,
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            ConditionalCompilation,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                ConditionalCompilation,
                            >(
                                &<ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                            0xD362E73A_u32,
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<
                                        ConditionalCompilation,
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
                    #[cfg(feature = "bar")]
                    Self::Constructor2(input) => {
                        if {
                            false
                                || {
                                    let constructor_0 = false;
                                    let constructor_0 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x9BAE9D5E_u32,
                                    >>::PAYABLE;
                                    constructor_0
                                }
                                || {
                                    let constructor_1 = false;
                                    #[cfg(feature = "foo")]
                                    let constructor_1 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0xD362E73A_u32,
                                    >>::PAYABLE;
                                    constructor_1
                                }
                                || {
                                    let constructor_2 = false;
                                    #[cfg(feature = "bar")]
                                    let constructor_2 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x92670C4C_u32,
                                    >>::PAYABLE;
                                    constructor_2
                                }
                                || {
                                    let constructor_3 = false;
                                    #[cfg(feature = "foo")]
                                    #[cfg(feature = "bar")]
                                    let constructor_3 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x8B85927D_u32,
                                    >>::PAYABLE;
                                    constructor_3
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0x92670C4C_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x92670C4C_u32,
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x92670C4C_u32,
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0x92670C4C_u32,
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            ConditionalCompilation,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                ConditionalCompilation,
                            >(
                                &<ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                            0x92670C4C_u32,
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<
                                        ConditionalCompilation,
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
                    #[cfg(feature = "foo")]
                    #[cfg(feature = "bar")]
                    Self::Constructor3(input) => {
                        if {
                            false
                                || {
                                    let constructor_0 = false;
                                    let constructor_0 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x9BAE9D5E_u32,
                                    >>::PAYABLE;
                                    constructor_0
                                }
                                || {
                                    let constructor_1 = false;
                                    #[cfg(feature = "foo")]
                                    let constructor_1 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0xD362E73A_u32,
                                    >>::PAYABLE;
                                    constructor_1
                                }
                                || {
                                    let constructor_2 = false;
                                    #[cfg(feature = "bar")]
                                    let constructor_2 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x92670C4C_u32,
                                    >>::PAYABLE;
                                    constructor_2
                                }
                                || {
                                    let constructor_3 = false;
                                    #[cfg(feature = "foo")]
                                    #[cfg(feature = "bar")]
                                    let constructor_3 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        0x8B85927D_u32,
                                    >>::PAYABLE;
                                    constructor_3
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0x8B85927D_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x8B85927D_u32,
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x8B85927D_u32,
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                0x8B85927D_u32,
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            ConditionalCompilation,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                ConditionalCompilation,
                            >(
                                &<ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                            0x8B85927D_u32,
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<
                                        ConditionalCompilation,
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
        impl ::ink::reflect::ContractConstructorDecoder for ConditionalCompilation {
            type Type = __ink_ConstructorDecoder;
        }
    };
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_MessageDecoder {
            #[cfg(feature = "foo")]
            Message0(
                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    0xC37844F9_u32,
                >>::Input,
            ),
            #[cfg(feature = "bar")]
            Message1(
                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    0x9F45AF95_u32,
                >>::Input,
            ),
            Message2(
                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x633AA551_u32,
                            >>::SELECTOR,
                        )
                    },
                >>::Input,
            ),
            Message3(
                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x2F865BD9_u32,
                            >>::SELECTOR,
                        )
                    },
                >>::Input,
            ),
            #[cfg(feature = "foo")]
            Message4(
                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x8212E40D_u32,
                            >>::SELECTOR,
                        )
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
                #[cfg(feature = "foo")]
                const MESSAGE_0: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    0xC37844F9_u32,
                >>::SELECTOR;
                #[cfg(feature = "bar")]
                const MESSAGE_1: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    0x9F45AF95_u32,
                >>::SELECTOR;
                const MESSAGE_2: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x633AA551_u32,
                            >>::SELECTOR,
                        )
                    },
                >>::SELECTOR;
                const MESSAGE_3: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x2F865BD9_u32,
                            >>::SELECTOR,
                        )
                    },
                >>::SELECTOR;
                #[cfg(feature = "foo")]
                const MESSAGE_4: [::core::primitive::u8; 4usize] = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x8212E40D_u32,
                            >>::SELECTOR,
                        )
                    },
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    #[cfg(feature = "foo")]
                    MESSAGE_0 => {
                        ::core::result::Result::Ok(
                            Self::Message0(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0xC37844F9_u32,
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    #[cfg(feature = "bar")]
                    MESSAGE_1 => {
                        ::core::result::Result::Ok(
                            Self::Message1(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0x9F45AF95_u32,
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
                                <<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_3 => {
                        ::core::result::Result::Ok(
                            Self::Message3(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x2F865BD9_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::Input as ::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    #[cfg(feature = "foo")]
                    MESSAGE_4 => {
                        ::core::result::Result::Ok(
                            Self::Message4(
                                <<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x8212E40D_u32,
                                            >>::SELECTOR,
                                        )
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
            contract: ::core::mem::ManuallyDrop<ConditionalCompilation>,
            mutates: bool,
        ) {
            if mutates {
                ::ink::env::set_contract_storage::<
                    ::ink::primitives::Key,
                    ConditionalCompilation,
                >(
                    &<ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
                    &contract,
                );
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_MessageDecoder {
            #[allow(clippy::nonminimal_bool, clippy::let_unit_value)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                let key = <ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY;
                let mut contract: ::core::mem::ManuallyDrop<ConditionalCompilation> = ::core::mem::ManuallyDrop::new(
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
                    #[cfg(feature = "foo")]
                    Self::Message0(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    #[cfg(feature = "foo")]
                                    let message_0 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0xC37844F9_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    #[cfg(feature = "bar")]
                                    let message_1 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0x9F45AF95_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x2F865BD9_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    #[cfg(feature = "foo")]
                                    let message_4 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x8212E40D_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                0xC37844F9_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            0xC37844F9_u32,
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            0xC37844F9_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0xC37844F9_u32,
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
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0xC37844F9_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0xC37844F9_u32,
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    #[cfg(feature = "bar")]
                    Self::Message1(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    #[cfg(feature = "foo")]
                                    let message_0 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0xC37844F9_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    #[cfg(feature = "bar")]
                                    let message_1 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0x9F45AF95_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x2F865BD9_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    #[cfg(feature = "foo")]
                                    let message_4 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x8212E40D_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                0x9F45AF95_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            0x9F45AF95_u32,
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            0x9F45AF95_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0x9F45AF95_u32,
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
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0x9F45AF95_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    0x9F45AF95_u32,
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    Self::Message2(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    #[cfg(feature = "foo")]
                                    let message_0 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0xC37844F9_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    #[cfg(feature = "bar")]
                                    let message_1 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0x9F45AF95_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x2F865BD9_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    #[cfg(feature = "foo")]
                                    let message_4 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x8212E40D_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                {
                                    ::core::primitive::u32::from_be_bytes(
                                        <<::ink::reflect::TraitDefinitionRegistry<
                                            <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                        > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                            0x633AA551_u32,
                                        >>::SELECTOR,
                                    )
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x633AA551_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x633AA551_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
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
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    Self::Message3(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    #[cfg(feature = "foo")]
                                    let message_0 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0xC37844F9_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    #[cfg(feature = "bar")]
                                    let message_1 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0x9F45AF95_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x2F865BD9_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    #[cfg(feature = "foo")]
                                    let message_4 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x8212E40D_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                {
                                    ::core::primitive::u32::from_be_bytes(
                                        <<::ink::reflect::TraitDefinitionRegistry<
                                            <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                        > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                            0x2F865BD9_u32,
                                        >>::SELECTOR,
                                    )
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x2F865BD9_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x2F865BD9_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x2F865BD9_u32,
                                            >>::SELECTOR,
                                        )
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
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x2F865BD9_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x2F865BD9_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::Output,
                            >,
                        >(
                            ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                            &::ink::MessageResult::Ok(result),
                        )
                    }
                    #[cfg(feature = "foo")]
                    Self::Message4(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    #[cfg(feature = "foo")]
                                    let message_0 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0xC37844F9_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    #[cfg(feature = "bar")]
                                    let message_1 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        0x9F45AF95_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x2F865BD9_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    #[cfg(feature = "foo")]
                                    let message_4 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x8212E40D_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                {
                                    ::core::primitive::u32::from_be_bytes(
                                        <<::ink::reflect::TraitDefinitionRegistry<
                                            <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                        > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                            0x8212E40D_u32,
                                        >>::SELECTOR,
                                    )
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x8212E40D_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::Output = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x8212E40D_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x8212E40D_u32,
                                            >>::SELECTOR,
                                        )
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
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x8212E40D_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x8212E40D_u32,
                                            >>::SELECTOR,
                                        )
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
        impl ::ink::reflect::ContractMessageDecoder for ConditionalCompilation {
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
                    || {
                        let constructor_0 = false;
                        let constructor_0 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::PAYABLE;
                        constructor_0
                    }
                    || {
                        let constructor_1 = false;
                        #[cfg(feature = "foo")]
                        let constructor_1 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0xD362E73A_u32,
                        >>::PAYABLE;
                        constructor_1
                    }
                    || {
                        let constructor_2 = false;
                        #[cfg(feature = "bar")]
                        let constructor_2 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x92670C4C_u32,
                        >>::PAYABLE;
                        constructor_2
                    }
                    || {
                        let constructor_3 = false;
                        #[cfg(feature = "foo")]
                        #[cfg(feature = "bar")]
                        let constructor_3 = <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                            0x8B85927D_u32,
                        >>::PAYABLE;
                        constructor_3
                    }
            } {
                ::ink::codegen::deny_payment::<
                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| ::core::panicking::panic_display(&error))
            }
            let dispatchable = match ::ink::env::decode_input::<
                <ConditionalCompilation as ::ink::reflect::ContractConstructorDecoder>::Type,
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
            <<ConditionalCompilation as ::ink::reflect::ContractConstructorDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
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
                    || {
                        let message_0 = false;
                        #[cfg(feature = "foo")]
                        let message_0 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            0xC37844F9_u32,
                        >>::PAYABLE;
                        message_0
                    }
                    || {
                        let message_1 = false;
                        #[cfg(feature = "bar")]
                        let message_1 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            0x9F45AF95_u32,
                        >>::PAYABLE;
                        message_1
                    }
                    || {
                        let message_2 = false;
                        let message_2 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x633AA551_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::PAYABLE;
                        message_2
                    }
                    || {
                        let message_3 = false;
                        let message_3 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x2F865BD9_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::PAYABLE;
                        message_3
                    }
                    || {
                        let message_4 = false;
                        #[cfg(feature = "foo")]
                        let message_4 = <ConditionalCompilation as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x8212E40D_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::PAYABLE;
                        message_4
                    }
            } {
                ::ink::codegen::deny_payment::<
                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| ::core::panicking::panic_display(&error))
            }
            let dispatchable = match ::ink::env::decode_input::<
                <ConditionalCompilation as ::ink::reflect::ContractMessageDecoder>::Type,
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
            <<ConditionalCompilation as ::ink::reflect::ContractMessageDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
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
        use ::ink::codegen::EmitEvent as _;
        const _: ::ink::codegen::utils::IsSameType<ConditionalCompilation> = ::ink::codegen::utils::IsSameType::<
            ConditionalCompilation,
        >::new();
        impl ConditionalCompilation {
            /// Creates a new flipper smart contract initialized to `false`.
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new() -> Self {
                Self { value: Default::default() }
            }
            /// Constructor that included when `foo` is enabled
            #[cfg(feature = "foo")]
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new_foo(value: bool) -> Self {
                Self { value }
            }
            /// Constructor that included when `bar` is enabled
            #[cfg(feature = "bar")]
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new_bar(value: bool) -> Self {
                Self { value }
            }
            /// Constructor that included with either `foo` or `bar` features enabled
            #[cfg(feature = "foo")]
            #[cfg(feature = "bar")]
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new_foo_bar(value: bool) -> Self {
                Self { value }
            }
            #[cfg(feature = "foo")]
            pub fn inherent_flip_foo(&mut self) {
                self.value = !self.value;
                let caller = Self::env().caller();
                Self::env()
                    .emit_event(Changes {
                        new_value: self.value,
                        by: caller,
                    });
            }
            #[cfg(feature = "bar")]
            pub fn inherent_flip_bar(&mut self) {
                let caller = Self::env().caller();
                let block_number = Self::env().block_number();
                self.value = !self.value;
                Self::env()
                    .emit_event(ChangesDated {
                        new_value: self.value,
                        by: caller,
                        when: block_number,
                    });
            }
        }
        const _: ::ink::codegen::utils::IsSameType<ConditionalCompilation> = ::ink::codegen::utils::IsSameType::<
            ConditionalCompilation,
        >::new();
        impl Flip for ConditionalCompilation {
            type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<
                Environment,
            > as Flip>::__ink_TraitInfo;
            type flipOutput = ();
            fn flip(&mut self) -> Self::flipOutput {
                self.value = !self.value;
            }
            type getOutput = bool;
            fn get(&self) -> Self::getOutput {
                self.value
            }
            #[cfg(feature = "foo")]
            type pushFooOutput = ();
            /// Feature gated mutating message
            #[cfg(feature = "foo")]
            fn push_foo(&mut self, value: bool) -> Self::pushFooOutput {
                let caller = Self::env().caller();
                Self::env()
                    .emit_event(Changes {
                        new_value: value,
                        by: caller,
                    });
                self.value = value;
            }
        }
        const _: () = {
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<bool>,
            >();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
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
                                "conditional_compilation::conditional_compilation",
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
            impl ::ink::codegen::ContractCallBuilder for ConditionalCompilation {
                type Type = CallBuilder;
            }
            impl ::ink::env::ContractEnv for CallBuilder {
                type Env = <ConditionalCompilation as ::ink::env::ContractEnv>::Env;
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
        #[doc(hidden)]
        impl ::ink::codegen::TraitCallForwarderFor<
            {
                <<::ink::reflect::TraitDefinitionRegistry<
                    Environment,
                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
            },
        > for CallBuilder {
            type Forwarder = <<Self as Flip>::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder;
            #[inline]
            fn forward(&self) -> &Self::Forwarder {
                unsafe {
                    &*(&self.account_id as *const AccountId as *const Self::Forwarder)
                }
            }
            #[inline]
            fn forward_mut(&mut self) -> &mut Self::Forwarder {
                unsafe {
                    &mut *(&mut self.account_id as *mut AccountId
                        as *mut Self::Forwarder)
                }
            }
            #[inline]
            fn build(
                &self,
            ) -> &<Self::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder {
                <_ as ::ink::codegen::TraitCallBuilder>::call(
                    <Self as ::ink::codegen::TraitCallForwarderFor<
                        {
                            <<::ink::reflect::TraitDefinitionRegistry<
                                Environment,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::forward(self),
                )
            }
            #[inline]
            fn build_mut(
                &mut self,
            ) -> &mut <Self::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder {
                <_ as ::ink::codegen::TraitCallBuilder>::call_mut(
                    <Self as ::ink::codegen::TraitCallForwarderFor<
                        {
                            <<::ink::reflect::TraitDefinitionRegistry<
                                Environment,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::forward_mut(self),
                )
            }
        }
        impl Flip for CallBuilder {
            type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<
                Environment,
            > as Flip>::__ink_TraitInfo;
            type flipOutput = <<<Self as ::ink::codegen::TraitCallForwarderFor<
                {
                    <<::ink::reflect::TraitDefinitionRegistry<
                        Environment,
                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                },
            >>::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder as Flip>::flipOutput;
            #[inline]
            fn flip(&mut self) -> Self::flipOutput {
                <_ as Flip>::flip(
                    <Self as ::ink::codegen::TraitCallForwarderFor<
                        {
                            <<::ink::reflect::TraitDefinitionRegistry<
                                Environment,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::build_mut(self),
                )
            }
            type getOutput = <<<Self as ::ink::codegen::TraitCallForwarderFor<
                {
                    <<::ink::reflect::TraitDefinitionRegistry<
                        Environment,
                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                },
            >>::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder as Flip>::getOutput;
            #[inline]
            fn get(&self) -> Self::getOutput {
                <_ as Flip>::get(
                    <Self as ::ink::codegen::TraitCallForwarderFor<
                        {
                            <<::ink::reflect::TraitDefinitionRegistry<
                                Environment,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::build(self),
                )
            }
            #[cfg(feature = "foo")]
            type pushFooOutput = <<<Self as ::ink::codegen::TraitCallForwarderFor<
                {
                    <<::ink::reflect::TraitDefinitionRegistry<
                        Environment,
                    > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                },
            >>::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder as Flip>::pushFooOutput;
            #[inline]
            /// Feature gated mutating message
            #[cfg(feature = "foo")]
            fn push_foo(&mut self, value: bool) -> Self::pushFooOutput {
                <_ as Flip>::push_foo(
                    <Self as ::ink::codegen::TraitCallForwarderFor<
                        {
                            <<::ink::reflect::TraitDefinitionRegistry<
                                Environment,
                            > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::build_mut(self),
                    value,
                )
            }
        }
        impl CallBuilder {
            #[cfg(feature = "foo")]
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn inherent_flip_foo(
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
                                0xC3_u8,
                                0x78_u8,
                                0x44_u8,
                                0xF9_u8,
                            ]),
                        ),
                    )
                    .returns::<()>()
            }
            #[cfg(feature = "bar")]
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn inherent_flip_bar(
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
                                0x9F_u8,
                                0x45_u8,
                                0xAF_u8,
                                0x95_u8,
                            ]),
                        ),
                    )
                    .returns::<()>()
            }
        }
    };
    pub struct ConditionalCompilationRef {
        inner: <ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ConditionalCompilationRef {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ConditionalCompilationRef",
                "inner",
                &&self.inner,
            )
        }
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for ConditionalCompilationRef {
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
        impl ::scale::EncodeLike for ConditionalCompilationRef {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Decode for ConditionalCompilationRef {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(ConditionalCompilationRef {
                    inner: {
                        let __codec_res_edqy = <<ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type as ::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e
                                        .chain(
                                            "Could not decode `ConditionalCompilationRef::inner`",
                                        ),
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
    impl ::core::hash::Hash for ConditionalCompilationRef {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.inner, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ConditionalCompilationRef {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ConditionalCompilationRef {
        #[inline]
        fn eq(&self, other: &ConditionalCompilationRef) -> bool {
            self.inner == other.inner
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for ConditionalCompilationRef {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ConditionalCompilationRef {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<
                <ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type,
            >;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ConditionalCompilationRef {
        #[inline]
        fn clone(&self) -> ConditionalCompilationRef {
            ConditionalCompilationRef {
                inner: ::core::clone::Clone::clone(&self.inner),
            }
        }
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::scale_info::TypeInfo for ConditionalCompilationRef {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new(
                            "ConditionalCompilationRef",
                            "conditional_compilation::conditional_compilation",
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        ::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type,
                                    >()
                                    .name("inner")
                                    .type_name(
                                        "<ConditionalCompilation as::ink::codegen::ContractCallBuilder>::Type",
                                    )
                            }),
                    )
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for ConditionalCompilationRef {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "ConditionalCompilationRef",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "inner",
                                <<ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type as ::ink::storage::traits::StorageLayout>::layout(
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
        impl ::ink::env::ContractReference for ConditionalCompilation {
            type Type = ConditionalCompilationRef;
        }
        impl ::ink::env::call::ConstructorReturnType<ConditionalCompilationRef>
        for ConditionalCompilation {
            type Output = ConditionalCompilationRef;
            type Error = ();
            fn ok(value: ConditionalCompilationRef) -> Self::Output {
                value
            }
        }
        impl<E> ::ink::env::call::ConstructorReturnType<ConditionalCompilationRef>
        for ::core::result::Result<ConditionalCompilation, E>
        where
            E: ::scale::Decode,
        {
            const IS_RESULT: bool = true;
            type Output = ::core::result::Result<ConditionalCompilationRef, E>;
            type Error = E;
            fn ok(value: ConditionalCompilationRef) -> Self::Output {
                ::core::result::Result::Ok(value)
            }
            fn err(err: Self::Error) -> ::core::option::Option<Self::Output> {
                ::core::option::Option::Some(::core::result::Result::Err(err))
            }
        }
        impl ::ink::env::ContractEnv for ConditionalCompilationRef {
            type Env = <ConditionalCompilation as ::ink::env::ContractEnv>::Env;
        }
    };
    impl Flip for ConditionalCompilationRef {
        type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<
            Environment,
        > as Flip>::__ink_TraitInfo;
        type flipOutput = <<Self::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder as Flip>::flipOutput;
        #[inline]
        fn flip(&mut self) -> Self::flipOutput {
            <_ as Flip>::flip(
                <_ as ::ink::codegen::TraitCallForwarderFor<
                    {
                        <<::ink::reflect::TraitDefinitionRegistry<
                            Environment,
                        > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                    },
                >>::forward_mut(
                    <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self),
                ),
            )
        }
        type getOutput = <<Self::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder as Flip>::getOutput;
        #[inline]
        fn get(&self) -> Self::getOutput {
            <_ as Flip>::get(
                <_ as ::ink::codegen::TraitCallForwarderFor<
                    {
                        <<::ink::reflect::TraitDefinitionRegistry<
                            Environment,
                        > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                    },
                >>::forward(<Self as ::ink::codegen::TraitCallBuilder>::call(self)),
            )
        }
        #[cfg(feature = "foo")]
        type pushFooOutput = <<Self::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder as Flip>::pushFooOutput;
        #[inline]
        #[cfg(feature = "foo")]
        fn push_foo(&mut self, value: bool) -> Self::pushFooOutput {
            <_ as Flip>::push_foo(
                <_ as ::ink::codegen::TraitCallForwarderFor<
                    {
                        <<::ink::reflect::TraitDefinitionRegistry<
                            Environment,
                        > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                    },
                >>::forward_mut(
                    <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self),
                ),
                value,
            )
        }
    }
    impl ConditionalCompilationRef {
        /// Creates a new flipper smart contract initialized to `false`.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new() -> ::ink::env::call::CreateBuilder<
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
                            0x9B_u8,
                            0xAE_u8,
                            0x9D_u8,
                            0x5E_u8,
                        ]),
                    ),
                )
                .returns::<Self>()
        }
        /// Constructor that included when `foo` is enabled
        #[cfg(feature = "foo")]
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new_foo(
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
                                0xD3_u8,
                                0x62_u8,
                                0xE7_u8,
                                0x3A_u8,
                            ]),
                        )
                        .push_arg(__ink_binding_0),
                )
                .returns::<Self>()
        }
        /// Constructor that included when `bar` is enabled
        #[cfg(feature = "bar")]
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new_bar(
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
                                0x92_u8,
                                0x67_u8,
                                0x0C_u8,
                                0x4C_u8,
                            ]),
                        )
                        .push_arg(__ink_binding_0),
                )
                .returns::<Self>()
        }
        /// Constructor that included with either `foo` or `bar` features enabled
        #[cfg(feature = "foo")]
        #[cfg(feature = "bar")]
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new_foo_bar(
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
                                0x8B_u8,
                                0x85_u8,
                                0x92_u8,
                                0x7D_u8,
                            ]),
                        )
                        .push_arg(__ink_binding_0),
                )
                .returns::<Self>()
        }
        #[cfg(feature = "foo")]
        #[inline]
        pub fn inherent_flip_foo(&mut self) {
            self.try_inherent_flip_foo()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "ConditionalCompilation", "inherent_flip_foo", error
                    ),
                ))
        }
        #[cfg(feature = "foo")]
        #[inline]
        pub fn try_inherent_flip_foo(&mut self) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .inherent_flip_foo()
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "ConditionalCompilation", "inherent_flip_foo", error
                    ),
                ))
        }
        #[cfg(feature = "bar")]
        #[inline]
        pub fn inherent_flip_bar(&mut self) {
            self.try_inherent_flip_bar()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "ConditionalCompilation", "inherent_flip_bar", error
                    ),
                ))
        }
        #[cfg(feature = "bar")]
        #[inline]
        pub fn try_inherent_flip_bar(&mut self) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .inherent_flip_bar()
                .try_invoke()
                .unwrap_or_else(|error| ::core::panicking::panic_fmt(
                    format_args!(
                        "encountered error while calling {0}::{1}: {2:?}",
                        "ConditionalCompilation", "inherent_flip_bar", error
                    ),
                ))
        }
    }
    const _: () = {
        impl ::ink::codegen::TraitCallBuilder for ConditionalCompilationRef {
            type Builder = <ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type;
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
    impl ::ink::env::call::FromAccountId<Environment> for ConditionalCompilationRef {
        #[inline]
        fn from_account_id(account_id: AccountId) -> Self {
            Self {
                inner: <<ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type as ::ink::env::call::FromAccountId<
                    Environment,
                >>::from_account_id(account_id),
            }
        }
    }
    impl ::ink::ToAccountId<Environment> for ConditionalCompilationRef {
        #[inline]
        fn to_account_id(&self) -> AccountId {
            <<ConditionalCompilation as ::ink::codegen::ContractCallBuilder>::Type as ::ink::ToAccountId<
                Environment,
            >>::to_account_id(&self.inner)
        }
    }
    impl ::core::convert::AsRef<AccountId> for ConditionalCompilationRef {
        fn as_ref(&self) -> &AccountId {
            <_ as ::core::convert::AsRef<AccountId>>::as_ref(&self.inner)
        }
    }
    impl ::core::convert::AsMut<AccountId> for ConditionalCompilationRef {
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
                    >>::from(
                        <ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
                    ),
                    <ConditionalCompilation as ::ink::storage::traits::StorageLayout>::layout(
                        &<ConditionalCompilation as ::ink::storage::traits::StorageKey>::KEY,
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
                            .args([])
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        2611912030u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
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
                        #[cfg(feature = "foo")]
                        ::ink::metadata::ConstructorSpec::from_label("new_foo")
                            .selector([0xD3_u8, 0x62_u8, 0xE7_u8, 0x3A_u8])
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
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        3546474298u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                                            3546474298u32,
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
                        #[cfg(feature = "bar")]
                        ::ink::metadata::ConstructorSpec::from_label("new_bar")
                            .selector([0x92_u8, 0x67_u8, 0x0C_u8, 0x4C_u8])
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
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        2456226892u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                                            2456226892u32,
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
                        #[cfg(feature = "foo")]
                        #[cfg(feature = "bar")]
                        ::ink::metadata::ConstructorSpec::from_label("new_foo_bar")
                            .selector([0x8B_u8, 0x85_u8, 0x92_u8, 0x7D_u8])
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
                            .payable(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                        2340786813u32,
                                    >>::IS_RESULT {
                                        ::core::option::Option::Some(
                                            ::ink::metadata::TypeSpec::with_name_str::<
                                                ::ink::ConstructorResult<
                                                    ::core::result::Result<
                                                        (),
                                                        <ConditionalCompilation as ::ink::reflect::DispatchableConstructorInfo<
                                                            2340786813u32,
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
                        #[cfg(feature = "foo")]
                        ::ink::metadata::MessageSpec::from_label("inherent_flip_foo")
                            .selector([0xC3_u8, 0x78_u8, 0x44_u8, 0xF9_u8])
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
                            .docs([])
                            .done(),
                        #[cfg(feature = "bar")]
                        ::ink::metadata::MessageSpec::from_label("inherent_flip_bar")
                            .selector([0x9F_u8, 0x45_u8, 0xAF_u8, 0x95_u8])
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
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("Flip::flip")
                            .selector({
                                <<::ink::reflect::TraitDefinitionRegistry<
                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                    0x633AA551_u32,
                                >>::SELECTOR
                            })
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
                            .payable({
                                <<::ink::reflect::TraitDefinitionRegistry<
                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                    0x633AA551_u32,
                                >>::PAYABLE
                            })
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("Flip::get")
                            .selector({
                                <<::ink::reflect::TraitDefinitionRegistry<
                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                    0x2F865BD9_u32,
                                >>::SELECTOR
                            })
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
                            .payable({
                                <<::ink::reflect::TraitDefinitionRegistry<
                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                    0x2F865BD9_u32,
                                >>::PAYABLE
                            })
                            .docs([])
                            .done(),
                        #[cfg(feature = "foo")]
                        ::ink::metadata::MessageSpec::from_label("Flip::push_foo")
                            .selector({
                                <<::ink::reflect::TraitDefinitionRegistry<
                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                    0x8212E40D_u32,
                                >>::SELECTOR
                            })
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
                            .payable({
                                <<::ink::reflect::TraitDefinitionRegistry<
                                    <ConditionalCompilation as ::ink::env::ContractEnv>::Env,
                                > as Flip>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                    0x8212E40D_u32,
                                >>::PAYABLE
                            })
                            .docs([])
                            .done(),
                    ])
                    .events([
                        #[cfg(feature = "foo")]
                        ::ink::metadata::EventSpec::new("Changes")
                            .args([
                                ::ink::metadata::EventParamSpec::new("new_value")
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
                                    .indexed(false)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new("by")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AccountId,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AccountId"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .indexed(true)
                                    .docs([])
                                    .done(),
                            ])
                            .docs([])
                            .done(),
                        #[cfg(feature = "bar")]
                        ::ink::metadata::EventSpec::new("ChangesDated")
                            .args([
                                ::ink::metadata::EventParamSpec::new("new_value")
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
                                    .indexed(false)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new("by")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AccountId,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AccountId"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .indexed(true)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new("when")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            BlockNumber,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["BlockNumber"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .indexed(false)
                                    .docs([])
                                    .done(),
                            ])
                            .docs([])
                            .done(),
                    ])
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
    use super::Flip;
}
