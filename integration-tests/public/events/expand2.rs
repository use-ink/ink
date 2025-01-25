#![feature(prelude_import)]
#![no_std]
#![no_main]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
#[codec(crate = ::ink::scale)]
#[ink(anonymous)]
pub struct AnonymousEvent {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::ink::scale::Decode for AnonymousEvent {
        fn decode<__CodecInputEdqy: ::ink::scale::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::ink::scale::Error> {
            ::core::result::Result::Ok(AnonymousEvent {
                topic: {
                    let __codec_res_edqy = <[u8; 32] as ::ink::scale::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `AnonymousEvent::topic`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
                field_1: {
                    let __codec_res_edqy = <u32 as ::ink::scale::Decode>::decode(
                        __codec_input_edqy,
                    );
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `AnonymousEvent::field_1`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                },
            })
        }
    }
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::ink::scale::Encode for AnonymousEvent {
        fn size_hint(&self) -> usize {
            0_usize
                .saturating_add(::ink::scale::Encode::size_hint(&self.topic))
                .saturating_add(::ink::scale::Encode::size_hint(&self.field_1))
        }
        fn encode_to<__CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            ::ink::scale::Encode::encode_to(&self.topic, __codec_dest_edqy);
            ::ink::scale::Encode::encode_to(&self.field_1, __codec_dest_edqy);
        }
    }
    #[automatically_derived]
    impl ::ink::scale::EncodeLike for AnonymousEvent {}
};
const _: () = {
    impl ::ink::env::Event for AnonymousEvent {
        type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];
        const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = ::core::option::Option::None;
        fn topics<E, B>(
            &self,
            builder: ::ink::env::event::TopicsBuilder<
                ::ink::env::event::state::Uninit,
                E,
                B,
            >,
        ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
        where
            E: ::ink::env::Environment,
            B: ::ink::env::event::TopicsBuilderBackend<E>,
        {
            match self {
                AnonymousEvent { topic: __binding_0, .. } => {
                    builder
                        .build::<Self>()
                        .push_topic({
                            #[allow(unused_imports)]
                            use ::ink::option_info::AsOptionFallback as _;
                            ::ink::option_info::AsOption(&__binding_0).value()
                        })
                        .finish()
                }
            }
        }
    }
};
pub mod events {
    impl ::ink::env::ContractEnv for Events {
        type Env = ::ink::env::DefaultEnvironment;
    }
    type Environment = <Events as ::ink::env::ContractEnv>::Env;
    type AccountId = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::AccountId;
    type Balance = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Balance;
    type Hash = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Hash;
    type Timestamp = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Timestamp;
    type BlockNumber = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::BlockNumber;
    type ChainExtension = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::ChainExtension;
    const MAX_EVENT_TOPICS: usize = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::MAX_EVENT_TOPICS;
    type EventRecord = <<Events as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::EventRecord;
    const _: () = {
        struct Check {
            salt: (),
            field_0: bool,
        }
    };
    #[cfg(not(target_vendor = "fortanix"))]
    pub struct Events {
        value: <bool as ::ink::storage::traits::AutoStorableHint<
            ::ink::storage::traits::ManualKey<2411655118u32, ()>,
        >>::Type,
    }
    const _: () = {
        impl<
            __ink_generic_salt: ::ink::storage::traits::StorageKey,
        > ::ink::storage::traits::StorableHint<__ink_generic_salt> for Events {
            type Type = Events;
            type PreferredKey = ::ink::storage::traits::AutoKey;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageKey for Events {
            const KEY: ::ink::primitives::Key = <() as ::ink::storage::traits::StorageKey>::KEY;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::Storable for Events {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::ink::scale::Input>(
                __input: &mut __ink_I,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(Events {
                    value: <<bool as ::ink::storage::traits::AutoStorableHint<
                        ::ink::storage::traits::ManualKey<2411655118u32, ()>,
                    >>::Type as ::ink::storage::traits::Storable>::decode(__input)?,
                })
            }
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encode<__ink_O: ::ink::scale::Output + ?::core::marker::Sized>(
                &self,
                __dest: &mut __ink_O,
            ) {
                match self {
                    Events { value: __binding_0 } => {
                        ::ink::storage::traits::Storable::encode(__binding_0, __dest);
                    }
                }
            }
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encoded_size(&self) -> ::core::primitive::usize {
                match self {
                    Events { value: __binding_0 } => {
                        ::core::primitive::usize::MIN
                            .saturating_add(
                                ::ink::storage::traits::Storable::encoded_size(__binding_0),
                            )
                    }
                }
            }
        }
    };
    const _: () = {
        impl ::ink::reflect::ContractName for Events {
            const NAME: &'static str = "Events";
        }
    };
    const _: () = {
        impl<'a> ::ink::codegen::Env for &'a Events {
            type EnvAccess = ::ink::EnvAccess<
                'a,
                <Events as ::ink::env::ContractEnv>::Env,
            >;
            fn env(self) -> Self::EnvAccess {
                <<Self as ::ink::codegen::Env>::EnvAccess as ::core::default::Default>::default()
            }
        }
        impl<'a> ::ink::codegen::StaticEnv for Events {
            type EnvAccess = ::ink::EnvAccess<
                'static,
                <Events as ::ink::env::ContractEnv>::Env,
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
    #[codec(crate = ::ink::scale)]
    pub struct InlineFlipped {
        value: bool,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Decode for InlineFlipped {
            fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(InlineFlipped {
                    value: {
                        let __codec_res_edqy = <bool as ::ink::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InlineFlipped::value`"),
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
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Encode for InlineFlipped {
            fn size_hint(&self) -> usize {
                ::ink::scale::Encode::size_hint(&&self.value)
            }
            fn encode_to<
                __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::ink::scale::Encode::encode_to(&&self.value, __codec_dest_edqy)
            }
            fn encode(&self) -> ::ink::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::ink::scale::Encode::encode(&&self.value)
            }
            fn using_encoded<
                __CodecOutputReturn,
                __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                        &[::core::primitive::u8],
                    ) -> __CodecOutputReturn,
            >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                ::ink::scale::Encode::using_encoded(&&self.value, f)
            }
        }
        #[automatically_derived]
        impl ::ink::scale::EncodeLike for InlineFlipped {}
    };
    const _: () = {
        impl ::ink::env::Event for InlineFlipped {
            type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];
            const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = ::core::option::Option::Some([
                0xAB_u8,
                0x76_u8,
                0x8E_u8,
                0xF7_u8,
                0x7E_u8,
                0x8F_u8,
                0xD0_u8,
                0xC0_u8,
                0x74_u8,
                0x0C_u8,
                0x52_u8,
                0x32_u8,
                0x52_u8,
                0xAE_u8,
                0xBD_u8,
                0x65_u8,
                0xD3_u8,
                0x7B_u8,
                0xB2_u8,
                0xB5_u8,
                0xF5_u8,
                0x08_u8,
                0xD2_u8,
                0x30_u8,
                0x6F_u8,
                0x87_u8,
                0x63_u8,
                0x96_u8,
                0xCA_u8,
                0x7E_u8,
                0x44_u8,
                0x47_u8,
            ]);
            fn topics<E, B>(
                &self,
                builder: ::ink::env::event::TopicsBuilder<
                    ::ink::env::event::state::Uninit,
                    E,
                    B,
                >,
            ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::event::TopicsBuilderBackend<E>,
            {
                match self {
                    InlineFlipped { .. } => {
                        builder
                            .build::<Self>()
                            .push_topic(Self::SIGNATURE_TOPIC.as_ref())
                            .finish()
                    }
                }
            }
        }
    };
    #[codec(crate = ::ink::scale)]
    #[ink(
        signature_topic = "1111111111111111111111111111111111111111111111111111111111111111"
    )]
    pub struct InlineCustomFlipped {
        value: bool,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Decode for InlineCustomFlipped {
            fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(InlineCustomFlipped {
                    value: {
                        let __codec_res_edqy = <bool as ::ink::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InlineCustomFlipped::value`"),
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
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Encode for InlineCustomFlipped {
            fn size_hint(&self) -> usize {
                ::ink::scale::Encode::size_hint(&&self.value)
            }
            fn encode_to<
                __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::ink::scale::Encode::encode_to(&&self.value, __codec_dest_edqy)
            }
            fn encode(&self) -> ::ink::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::ink::scale::Encode::encode(&&self.value)
            }
            fn using_encoded<
                __CodecOutputReturn,
                __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                        &[::core::primitive::u8],
                    ) -> __CodecOutputReturn,
            >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                ::ink::scale::Encode::using_encoded(&&self.value, f)
            }
        }
        #[automatically_derived]
        impl ::ink::scale::EncodeLike for InlineCustomFlipped {}
    };
    const _: () = {
        impl ::ink::env::Event for InlineCustomFlipped {
            type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];
            const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = ::core::option::Option::Some([
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
                17u8,
            ]);
            fn topics<E, B>(
                &self,
                builder: ::ink::env::event::TopicsBuilder<
                    ::ink::env::event::state::Uninit,
                    E,
                    B,
                >,
            ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::event::TopicsBuilderBackend<E>,
            {
                match self {
                    InlineCustomFlipped { .. } => {
                        builder
                            .build::<Self>()
                            .push_topic(Self::SIGNATURE_TOPIC.as_ref())
                            .finish()
                    }
                }
            }
        }
    };
    #[codec(crate = ::ink::scale)]
    #[ink(anonymous)]
    pub struct InlineAnonymousEvent {
        #[ink(topic)]
        pub topic: [u8; 32],
        pub field_1: u32,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Decode for InlineAnonymousEvent {
            fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(InlineAnonymousEvent {
                    topic: {
                        let __codec_res_edqy = <[u8; 32] as ::ink::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InlineAnonymousEvent::topic`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                    field_1: {
                        let __codec_res_edqy = <u32 as ::ink::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `InlineAnonymousEvent::field_1`"),
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
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Encode for InlineAnonymousEvent {
            fn size_hint(&self) -> usize {
                0_usize
                    .saturating_add(::ink::scale::Encode::size_hint(&self.topic))
                    .saturating_add(::ink::scale::Encode::size_hint(&self.field_1))
            }
            fn encode_to<
                __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::ink::scale::Encode::encode_to(&self.topic, __codec_dest_edqy);
                ::ink::scale::Encode::encode_to(&self.field_1, __codec_dest_edqy);
            }
        }
        #[automatically_derived]
        impl ::ink::scale::EncodeLike for InlineAnonymousEvent {}
    };
    const _: () = {
        impl ::ink::env::Event for InlineAnonymousEvent {
            type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];
            const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> = ::core::option::Option::None;
            fn topics<E, B>(
                &self,
                builder: ::ink::env::event::TopicsBuilder<
                    ::ink::env::event::state::Uninit,
                    E,
                    B,
                >,
            ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
            where
                E: ::ink::env::Environment,
                B: ::ink::env::event::TopicsBuilderBackend<E>,
            {
                match self {
                    InlineAnonymousEvent { topic: __binding_0, .. } => {
                        builder
                            .build::<Self>()
                            .push_topic({
                                #[allow(unused_imports)]
                                use ::ink::option_info::AsOptionFallback as _;
                                ::ink::option_info::AsOption(&__binding_0).value()
                            })
                            .finish()
                    }
                }
            }
        }
    };
    impl ::ink::reflect::DispatchableConstructorInfo<0x9BAE9D5E_u32> for Events {
        type Input = bool;
        type Output = Self;
        type Storage = Events;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Events>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Events>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |__ink_binding_0| {
            Events::new(__ink_binding_0)
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
    impl ::ink::reflect::DispatchableMessageInfo<0x7F167334_u32> for Events {
        type Input = ();
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { Events::flip_with_foreign_event(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x7F_u8,
            0x16_u8,
            0x73_u8,
            0x34_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "flip_with_foreign_event";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0xDDF6121E_u32> for Events {
        type Input = ();
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { Events::flip_with_inline_event(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xDD_u8,
            0xF6_u8,
            0x12_u8,
            0x1E_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "flip_with_inline_event";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0xB7771685_u32> for Events {
        type Input = ();
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { Events::flip_with_inline_custom_event(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xB7_u8,
            0x77_u8,
            0x16_u8,
            0x85_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "flip_with_inline_custom_event";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0xEB243827_u32> for Events {
        type Input = Option<[u8; 32]>;
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { Events::emit_32_byte_topic_event(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xEB_u8,
            0x24_u8,
            0x38_u8,
            0x27_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "emit_32_byte_topic_event";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x3C111980_u32> for Events {
        type Input = Option<[u8; 32]>;
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { Events::emit_event_from_a_different_crate(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x3C_u8,
            0x11_u8,
            0x19_u8,
            0x80_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "emit_event_from_a_different_crate";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x889DE210_u32> for Events {
        type Input = [u8; 32];
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { Events::emit_anonymous_events(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x88_u8,
            0x9D_u8,
            0xE2_u8,
            0x10_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "emit_anonymous_events";
    }
    impl ::ink::reflect::DispatchableMessageInfo<
        {
            ::core::primitive::u32::from_be_bytes({
                <<::ink::reflect::TraitDefinitionRegistry<
                    <Events as ::ink::env::ContractEnv>::Env,
                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                    0x633AA551_u32,
                >>::SELECTOR
            })
        },
    > for Events {
        type Input = ();
        type Output = ();
        type Storage = Events;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            _|
        { <Events as event_def_unused::FlipperTrait>::flip(storage) };
        const SELECTOR: [::core::primitive::u8; 4usize] = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <Events as ::ink::env::ContractEnv>::Env,
            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x633AA551_u32,
            >>::SELECTOR
        };
        const PAYABLE: ::core::primitive::bool = {
            <<::ink::reflect::TraitDefinitionRegistry<
                <Events as ::ink::env::ContractEnv>::Env,
            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                0x633AA551_u32,
            >>::PAYABLE
        };
        const MUTATES: ::core::primitive::bool = true;
        const LABEL: &'static ::core::primitive::str = "FlipperTrait::flip";
    }
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_ConstructorDecoder {
            Constructor0(
                <Events as ::ink::reflect::DispatchableConstructorInfo<
                    0x9BAE9D5E_u32,
                >>::Input,
            ),
        }
        impl ::ink::reflect::DecodeDispatch for __ink_ConstructorDecoder {
            fn decode_dispatch<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
            where
                I: ::ink::scale::Input,
            {
                const CONSTRUCTOR_0: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableConstructorInfo<
                    0x9BAE9D5E_u32,
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::ink::scale::Decode>::decode(
                        input,
                    )
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    CONSTRUCTOR_0 => {
                        ::core::result::Result::Ok(
                            Self::Constructor0(
                                <<Events as ::ink::reflect::DispatchableConstructorInfo<
                                    0x9BAE9D5E_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
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
        impl ::ink::scale::Decode for __ink_ConstructorDecoder {
            fn decode<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::scale::Error>
            where
                I: ::ink::scale::Input,
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
                                    let constructor_0 = <Events as ::ink::reflect::DispatchableConstructorInfo<
                                        0x9BAE9D5E_u32,
                                    >>::PAYABLE;
                                    constructor_0
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableConstructorInfo<
                                0x9BAE9D5E_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <Events as ::ink::reflect::DispatchableConstructorInfo<
                                0x9BAE9D5E_u32,
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            Events,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract)
                            = output_result.as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                Events,
                            >(
                                &<Events as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        let mut flag = ::ink::env::ReturnFlags::empty();
                        if output_result.is_err() {
                            flag = ::ink::env::ReturnFlags::REVERT;
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <Events as ::ink::reflect::DispatchableConstructorInfo<
                                            0x9BAE9D5E_u32,
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<Events>>::Error,
                                >,
                            >,
                        >(
                            flag,
                            &::ink::ConstructorResult::Ok(output_result.map(|_| ())),
                        );
                    }
                }
            }
        }
        impl ::ink::reflect::ContractConstructorDecoder for Events {
            type Type = __ink_ConstructorDecoder;
        }
    };
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_MessageDecoder {
            Message0(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    0x7F167334_u32,
                >>::Input,
            ),
            Message1(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    0xDDF6121E_u32,
                >>::Input,
            ),
            Message2(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    0xB7771685_u32,
                >>::Input,
            ),
            Message3(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    0xEB243827_u32,
                >>::Input,
            ),
            Message4(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    0x3C111980_u32,
                >>::Input,
            ),
            Message5(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    0x889DE210_u32,
                >>::Input,
            ),
            Message6(
                <Events as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <Events as ::ink::env::ContractEnv>::Env,
                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x633AA551_u32,
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
                I: ::ink::scale::Input,
            {
                const MESSAGE_0: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    0x7F167334_u32,
                >>::SELECTOR;
                const MESSAGE_1: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    0xDDF6121E_u32,
                >>::SELECTOR;
                const MESSAGE_2: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    0xB7771685_u32,
                >>::SELECTOR;
                const MESSAGE_3: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    0xEB243827_u32,
                >>::SELECTOR;
                const MESSAGE_4: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    0x3C111980_u32,
                >>::SELECTOR;
                const MESSAGE_5: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    0x889DE210_u32,
                >>::SELECTOR;
                const MESSAGE_6: [::core::primitive::u8; 4usize] = <Events as ::ink::reflect::DispatchableMessageInfo<
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink::reflect::TraitDefinitionRegistry<
                                <Events as ::ink::env::ContractEnv>::Env,
                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                0x633AA551_u32,
                            >>::SELECTOR,
                        )
                    },
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::ink::scale::Decode>::decode(
                        input,
                    )
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    MESSAGE_0 => {
                        ::core::result::Result::Ok(
                            Self::Message0(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x7F167334_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_1 => {
                        ::core::result::Result::Ok(
                            Self::Message1(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xDDF6121E_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_2 => {
                        ::core::result::Result::Ok(
                            Self::Message2(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xB7771685_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_3 => {
                        ::core::result::Result::Ok(
                            Self::Message3(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xEB243827_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_4 => {
                        ::core::result::Result::Ok(
                            Self::Message4(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x3C111980_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_5 => {
                        ::core::result::Result::Ok(
                            Self::Message5(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x889DE210_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_6 => {
                        ::core::result::Result::Ok(
                            Self::Message6(
                                <<Events as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <Events as ::ink::env::ContractEnv>::Env,
                                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::Input as ::ink::scale::Decode>::decode(input)
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
        impl ::ink::scale::Decode for __ink_MessageDecoder {
            fn decode<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::scale::Error>
            where
                I: ::ink::scale::Input,
            {
                <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                    .map_err(::core::convert::Into::into)
            }
        }
        fn push_contract(contract: ::core::mem::ManuallyDrop<Events>, mutates: bool) {
            if mutates {
                ::ink::env::set_contract_storage::<
                    ::ink::primitives::Key,
                    Events,
                >(&<Events as ::ink::storage::traits::StorageKey>::KEY, &contract);
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_MessageDecoder {
            #[allow(clippy::nonminimal_bool, clippy::let_unit_value)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                let key = <Events as ::ink::storage::traits::StorageKey>::KEY;
                let mut contract: ::core::mem::ManuallyDrop<Events> = ::core::mem::ManuallyDrop::new(
                    match ::ink::env::get_contract_storage(&key) {
                        ::core::result::Result::Ok(
                            ::core::option::Option::Some(value),
                        ) => value,
                        ::core::result::Result::Ok(::core::option::Option::None) => {
                            ::core::panicking::panic_fmt(
                                format_args!("storage entry was empty"),
                            );
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::panicking::panic_fmt(
                                format_args!("could not properly decode storage entry"),
                            );
                        }
                    },
                );
                match self {
                    Self::Message0(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                0x7F167334_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x7F167334_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x7F167334_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x7F167334_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x7F167334_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x7F167334_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message1(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                0xDDF6121E_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xDDF6121E_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xDDF6121E_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xDDF6121E_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xDDF6121E_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xDDF6121E_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message2(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                0xB7771685_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xB7771685_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xB7771685_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xB7771685_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xB7771685_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xB7771685_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message3(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                0xEB243827_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xEB243827_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xEB243827_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xEB243827_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xEB243827_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0xEB243827_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message4(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                0x3C111980_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x3C111980_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x3C111980_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x3C111980_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x3C111980_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x3C111980_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message5(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                0x889DE210_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x889DE210_u32,
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x889DE210_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x889DE210_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x889DE210_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    0x889DE210_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message6(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x7F167334_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xDDF6121E_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xB7771685_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0xEB243827_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x3C111980_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                                || {
                                    let message_5 = false;
                                    let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        0x889DE210_u32,
                                    >>::PAYABLE;
                                    message_5
                                }
                                || {
                                    let message_6 = false;
                                    let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                                        {
                                            ::core::primitive::u32::from_be_bytes(
                                                <<::ink::reflect::TraitDefinitionRegistry<
                                                    <Events as ::ink::env::ContractEnv>::Env,
                                                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                    0x633AA551_u32,
                                                >>::SELECTOR,
                                            )
                                        },
                                    >>::PAYABLE;
                                    message_6
                                }
                        }
                            && !<Events as ::ink::reflect::DispatchableMessageInfo<
                                {
                                    ::core::primitive::u32::from_be_bytes(
                                        <<::ink::reflect::TraitDefinitionRegistry<
                                            <Events as ::ink::env::ContractEnv>::Env,
                                        > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                            0x633AA551_u32,
                                        >>::SELECTOR,
                                    )
                                },
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Events as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Events as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <Events as ::ink::env::ContractEnv>::Env,
                                    > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x633AA551_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::Output = <Events as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <Events as ::ink::env::ContractEnv>::Env,
                                    > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x633AA551_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <Events as ::ink::env::ContractEnv>::Env,
                                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
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
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <Events as ::ink::env::ContractEnv>::Env,
                                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Events as ::ink::reflect::DispatchableMessageInfo<
                                    {
                                        ::core::primitive::u32::from_be_bytes(
                                            <<::ink::reflect::TraitDefinitionRegistry<
                                                <Events as ::ink::env::ContractEnv>::Env,
                                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                                0x633AA551_u32,
                                            >>::SELECTOR,
                                        )
                                    },
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                };
            }
        }
        impl ::ink::reflect::ContractMessageDecoder for Events {
            type Type = __ink_MessageDecoder;
        }
    };
    #[cfg(not(any(test, feature = "std", feature = "ink-as-dependency")))]
    mod __do_not_access__ {
        use super::*;
        #[allow(dead_code)]
        #[allow(clippy::nonminimal_bool)]
        #[cfg(target_arch = "riscv64")]
        fn internal_deploy() {
            if !{
                false
                    || {
                        let constructor_0 = false;
                        let constructor_0 = <Events as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::PAYABLE;
                        constructor_0
                    }
            } {
                ::ink::codegen::deny_payment::<
                    <Events as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| {
                        #[cold]
                        #[track_caller]
                        #[inline(never)]
                        #[rustc_const_panic_str]
                        #[rustc_do_not_const_check]
                        const fn panic_cold_display<T: ::core::fmt::Display>(
                            arg: &T,
                        ) -> ! {
                            ::core::panicking::panic_display(arg)
                        }
                        panic_cold_display(&error);
                    })
            }
            let dispatchable = match ::ink::env::decode_input::<
                <Events as ::ink::reflect::ContractConstructorDecoder>::Type,
            >() {
                ::core::result::Result::Ok(decoded_dispatchable) => decoded_dispatchable,
                ::core::result::Result::Err(_decoding_error) => {
                    let error = ::ink::ConstructorResult::Err(
                        ::ink::LangError::CouldNotReadInput,
                    );
                    ::ink::env::return_value::<
                        ::ink::ConstructorResult<()>,
                    >(::ink::env::ReturnFlags::REVERT, &error);
                }
            };
            <<Events as ::ink::reflect::ContractConstructorDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
                    dispatchable,
                )
                .unwrap_or_else(|error| {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "dispatching ink! constructor failed: {0}", error
                            ),
                        );
                    }
                })
        }
        #[allow(dead_code)]
        #[allow(clippy::nonminimal_bool)]
        #[cfg(target_arch = "riscv64")]
        fn internal_call() {
            if !{
                false
                    || {
                        let message_0 = false;
                        let message_0 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x7F167334_u32,
                        >>::PAYABLE;
                        message_0
                    }
                    || {
                        let message_1 = false;
                        let message_1 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xDDF6121E_u32,
                        >>::PAYABLE;
                        message_1
                    }
                    || {
                        let message_2 = false;
                        let message_2 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xB7771685_u32,
                        >>::PAYABLE;
                        message_2
                    }
                    || {
                        let message_3 = false;
                        let message_3 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0xEB243827_u32,
                        >>::PAYABLE;
                        message_3
                    }
                    || {
                        let message_4 = false;
                        let message_4 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x3C111980_u32,
                        >>::PAYABLE;
                        message_4
                    }
                    || {
                        let message_5 = false;
                        let message_5 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            0x889DE210_u32,
                        >>::PAYABLE;
                        message_5
                    }
                    || {
                        let message_6 = false;
                        let message_6 = <Events as ::ink::reflect::DispatchableMessageInfo<
                            {
                                ::core::primitive::u32::from_be_bytes(
                                    <<::ink::reflect::TraitDefinitionRegistry<
                                        <Events as ::ink::env::ContractEnv>::Env,
                                    > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitMessageInfo<
                                        0x633AA551_u32,
                                    >>::SELECTOR,
                                )
                            },
                        >>::PAYABLE;
                        message_6
                    }
            } {
                ::ink::codegen::deny_payment::<
                    <Events as ::ink::env::ContractEnv>::Env,
                >()
                    .unwrap_or_else(|error| {
                        #[cold]
                        #[track_caller]
                        #[inline(never)]
                        #[rustc_const_panic_str]
                        #[rustc_do_not_const_check]
                        const fn panic_cold_display<T: ::core::fmt::Display>(
                            arg: &T,
                        ) -> ! {
                            ::core::panicking::panic_display(arg)
                        }
                        panic_cold_display(&error);
                    })
            }
            let dispatchable = match ::ink::env::decode_input::<
                <Events as ::ink::reflect::ContractMessageDecoder>::Type,
            >() {
                ::core::result::Result::Ok(decoded_dispatchable) => decoded_dispatchable,
                ::core::result::Result::Err(_decoding_error) => {
                    let error = ::ink::MessageResult::Err(
                        ::ink::LangError::CouldNotReadInput,
                    );
                    ::ink::env::return_value::<
                        ::ink::MessageResult<()>,
                    >(::ink::env::ReturnFlags::REVERT, &error);
                }
            };
            <<Events as ::ink::reflect::ContractMessageDecoder>::Type as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(
                    dispatchable,
                )
                .unwrap_or_else(|error| {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!("dispatching ink! message failed: {0}", error),
                        );
                    }
                })
        }
        #[cfg(target_arch = "riscv64")]
        pub fn call() {
            internal_call()
        }
        #[cfg(target_arch = "riscv64")]
        pub fn deploy() {
            internal_deploy()
        }
    }
    const _: () = {
        use ::ink::codegen::{Env as _, StaticEnv as _};
        const _: ::ink::codegen::utils::IsSameType<Events> = ::ink::codegen::utils::IsSameType::<
            Events,
        >::new();
        impl Events {
            /// Creates a new events smart contract initialized with the given value.
            #[cfg(not(target_os = "dragonfly"))]
            pub fn new(init_value: bool) -> Self {
                Self { value: init_value }
            }
            /// Flips the current value of the boolean.
            pub fn flip_with_foreign_event(&mut self) {
                self.value = !self.value;
                self.env()
                    .emit_event(event_def::ForeignFlipped {
                        value: self.value,
                    })
            }
            /// Flips the current value of the boolean.
            pub fn flip_with_inline_event(&mut self) {
                self.value = !self.value;
                self.env().emit_event(InlineFlipped { value: self.value })
            }
            /// Flips the current value of the boolean.
            pub fn flip_with_inline_custom_event(&mut self) {
                self.value = !self.value;
                self.env()
                    .emit_event(InlineCustomFlipped {
                        value: self.value,
                    })
            }
            /// Emit an event with a 32 byte topic.
            pub fn emit_32_byte_topic_event(&self, maybe_hash: Option<[u8; 32]>) {
                self.env()
                    .emit_event(event_def::ThirtyTwoByteTopics {
                        hash: [0x42; 32],
                        maybe_hash,
                    })
            }
            /// Emit an event from a different crate.
            pub fn emit_event_from_a_different_crate(
                &self,
                maybe_hash: Option<[u8; 32]>,
            ) {
                self.env()
                    .emit_event(event_def2::EventDefAnotherCrate {
                        hash: [0x42; 32],
                        maybe_hash,
                    })
            }
            /// Emit a inline and standalone anonymous events
            pub fn emit_anonymous_events(&self, topic: [u8; 32]) {
                self.env()
                    .emit_event(InlineAnonymousEvent {
                        topic,
                        field_1: 42,
                    });
                self.env()
                    .emit_event(super::AnonymousEvent {
                        topic,
                        field_1: 42,
                    });
            }
        }
        const _: ::ink::codegen::utils::IsSameType<Events> = ::ink::codegen::utils::IsSameType::<
            Events,
        >::new();
        /// Implementing the trait from the `event_def_unused` crate includes all defined
        /// events there.
        impl event_def_unused::FlipperTrait for Events {
            type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<
                Environment,
            > as event_def_unused::FlipperTrait>::__ink_TraitInfo;
            type flipOutput = ();
            fn flip(&mut self) -> Self::flipOutput {
                self.value = !self.value;
            }
        }
        const _: () = {
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<bool>>();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<Option<[u8; 32]>>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<Option<[u8; 32]>>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<[u8; 32]>,
            >();
        };
    };
    const _: () = {
        #[codec(crate = ::ink::scale)]
        /// The ink! smart contract's call builder.
        ///
        /// Implements the underlying on-chain calling of the ink! smart contract
        /// messages and trait implementations in a type safe way.
        #[repr(transparent)]
        pub struct CallBuilder {
            addr: ::ink::H160,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::ink::scale::Decode for CallBuilder {
                fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                    ::core::result::Result::Ok(CallBuilder {
                        addr: {
                            let __codec_res_edqy = <::ink::H160 as ::ink::scale::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `CallBuilder::addr`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
                }
                fn decode_into<__CodecInputEdqy: ::ink::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                    dst_: &mut ::core::mem::MaybeUninit<Self>,
                ) -> ::core::result::Result<
                    ::ink::scale::DecodeFinished,
                    ::ink::scale::Error,
                > {
                    match (
                        &::core::mem::size_of::<::ink::H160>(),
                        &::core::mem::size_of::<Self>(),
                    ) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                let kind = ::core::panicking::AssertKind::Eq;
                                ::core::panicking::assert_failed(
                                    kind,
                                    &*left_val,
                                    &*right_val,
                                    ::core::option::Option::None,
                                );
                            }
                        }
                    };
                    if !(if ::core::mem::size_of::<::ink::H160>() > 0 { 1 } else { 0 }
                        <= 1)
                    {
                        ::core::panicking::panic(
                            "assertion failed: if ::core::mem::size_of::<::ink::H160>() > 0 { 1 } else { 0 } <= 1",
                        )
                    }
                    {
                        let dst_: &mut ::core::mem::MaybeUninit<Self> = dst_;
                        let dst_: &mut ::core::mem::MaybeUninit<::ink::H160> = unsafe {
                            &mut *dst_
                                .as_mut_ptr()
                                .cast::<::core::mem::MaybeUninit<::ink::H160>>()
                        };
                        <::ink::H160 as ::ink::scale::Decode>::decode_into(
                            __codec_input_edqy,
                            dst_,
                        )?;
                    }
                    unsafe {
                        ::core::result::Result::Ok(
                            ::ink::scale::DecodeFinished::assert_decoding_finished(),
                        )
                    }
                }
            }
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::ink::scale::Encode for CallBuilder {
                fn size_hint(&self) -> usize {
                    ::ink::scale::Encode::size_hint(&&self.addr)
                }
                fn encode_to<
                    __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    ::ink::scale::Encode::encode_to(&&self.addr, __codec_dest_edqy)
                }
                fn encode(
                    &self,
                ) -> ::ink::scale::alloc::vec::Vec<::core::primitive::u8> {
                    ::ink::scale::Encode::encode(&&self.addr)
                }
                fn using_encoded<
                    __CodecOutputReturn,
                    __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                            &[::core::primitive::u8],
                        ) -> __CodecOutputReturn,
                >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                    ::ink::scale::Encode::using_encoded(&&self.addr, f)
                }
            }
            #[automatically_derived]
            impl ::ink::scale::EncodeLike for CallBuilder {}
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for CallBuilder {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "CallBuilder",
                    "addr",
                    &&self.addr,
                )
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for CallBuilder {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.addr, state)
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CallBuilder {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CallBuilder {
            #[inline]
            fn eq(&self, other: &CallBuilder) -> bool {
                self.addr == other.addr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for CallBuilder {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<::ink::H160>;
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CallBuilder {
            #[inline]
            fn clone(&self) -> CallBuilder {
                CallBuilder {
                    addr: ::core::clone::Clone::clone(&self.addr),
                }
            }
        }
        const _: () = {
            impl ::ink::codegen::ContractCallBuilder for Events {
                type Type = CallBuilder;
            }
            impl ::ink::env::ContractEnv for CallBuilder {
                type Env = <Events as ::ink::env::ContractEnv>::Env;
            }
        };
        impl ::ink::env::call::FromAddr for CallBuilder {
            #[inline]
            fn from_addr(addr: ::ink::H160) -> Self {
                Self { addr }
            }
        }
        impl ::ink::ToAddr for CallBuilder {
            #[inline]
            fn to_addr(&self) -> ::ink::H160 {
                <::ink::H160 as ::core::clone::Clone>::clone(&self.addr)
            }
        }
        impl ::core::convert::AsRef<::ink::H160> for CallBuilder {
            fn as_ref(&self) -> &::ink::H160 {
                &self.addr
            }
        }
        impl ::core::convert::AsMut<::ink::H160> for CallBuilder {
            fn as_mut(&mut self) -> &mut ::ink::H160 {
                &mut self.addr
            }
        }
        #[doc(hidden)]
        impl ::ink::codegen::TraitCallForwarderFor<
            {
                <<::ink::reflect::TraitDefinitionRegistry<
                    Environment,
                > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
            },
        > for CallBuilder {
            type Forwarder = <<Self as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder;
            #[inline]
            fn forward(&self) -> &Self::Forwarder {
                unsafe { &*(&self.addr as *const ::ink::H160 as *const Self::Forwarder) }
            }
            #[inline]
            fn forward_mut(&mut self) -> &mut Self::Forwarder {
                unsafe {
                    &mut *(&mut self.addr as *mut ::ink::H160 as *mut Self::Forwarder)
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
                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
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
                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::forward_mut(self),
                )
            }
        }
        impl event_def_unused::FlipperTrait for CallBuilder {
            type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<
                Environment,
            > as event_def_unused::FlipperTrait>::__ink_TraitInfo;
            type flipOutput = <<<Self as ::ink::codegen::TraitCallForwarderFor<
                {
                    <<::ink::reflect::TraitDefinitionRegistry<
                        Environment,
                    > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                },
            >>::Forwarder as ::ink::codegen::TraitCallBuilder>::Builder as event_def_unused::FlipperTrait>::flipOutput;
            #[inline]
            fn flip(&mut self) -> Self::flipOutput {
                <_ as event_def_unused::FlipperTrait>::flip(
                    <Self as ::ink::codegen::TraitCallForwarderFor<
                        {
                            <<::ink::reflect::TraitDefinitionRegistry<
                                Environment,
                            > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                        },
                    >>::build_mut(self),
                )
            }
        }
        impl CallBuilder {
            /// Flips the current value of the boolean.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn flip_with_foreign_event(
                &mut self,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAddr::to_addr(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0x7F_u8,
                                0x16_u8,
                                0x73_u8,
                                0x34_u8,
                            ]),
                        ),
                    )
                    .returns::<()>()
            }
            /// Flips the current value of the boolean.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn flip_with_inline_event(
                &mut self,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAddr::to_addr(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0xDD_u8,
                                0xF6_u8,
                                0x12_u8,
                                0x1E_u8,
                            ]),
                        ),
                    )
                    .returns::<()>()
            }
            /// Flips the current value of the boolean.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn flip_with_inline_custom_event(
                &mut self,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAddr::to_addr(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([
                                0xB7_u8,
                                0x77_u8,
                                0x16_u8,
                                0x85_u8,
                            ]),
                        ),
                    )
                    .returns::<()>()
            }
            /// Emit an event with a 32 byte topic.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn emit_32_byte_topic_event(
                &self,
                __ink_binding_0: Option<[u8; 32]>,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<Option<[u8; 32]>>,
                            ::ink::env::call::utils::EmptyArgumentList,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAddr::to_addr(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0xEB_u8,
                                    0x24_u8,
                                    0x38_u8,
                                    0x27_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<()>()
            }
            /// Emit an event from a different crate.
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn emit_event_from_a_different_crate(
                &self,
                __ink_binding_0: Option<[u8; 32]>,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<Option<[u8; 32]>>,
                            ::ink::env::call::utils::EmptyArgumentList,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAddr::to_addr(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0x3C_u8,
                                    0x11_u8,
                                    0x19_u8,
                                    0x80_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<()>()
            }
            /// Emit a inline and standalone anonymous events
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn emit_anonymous_events(
                &self,
                __ink_binding_0: [u8; 32],
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<[u8; 32]>,
                            ::ink::env::call::utils::EmptyArgumentList,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<()>>,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAddr::to_addr(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0x88_u8,
                                    0x9D_u8,
                                    0xE2_u8,
                                    0x10_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<()>()
            }
        }
    };
    #[codec(crate = ::ink::scale)]
    pub struct EventsRef {
        inner: <Events as ::ink::codegen::ContractCallBuilder>::Type,
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Decode for EventsRef {
            fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(EventsRef {
                    inner: {
                        let __codec_res_edqy = <<Events as ::ink::codegen::ContractCallBuilder>::Type as ::ink::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `EventsRef::inner`"),
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
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Encode for EventsRef {
            fn size_hint(&self) -> usize {
                ::ink::scale::Encode::size_hint(&&self.inner)
            }
            fn encode_to<
                __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::ink::scale::Encode::encode_to(&&self.inner, __codec_dest_edqy)
            }
            fn encode(&self) -> ::ink::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::ink::scale::Encode::encode(&&self.inner)
            }
            fn using_encoded<
                __CodecOutputReturn,
                __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                        &[::core::primitive::u8],
                    ) -> __CodecOutputReturn,
            >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                ::ink::scale::Encode::using_encoded(&&self.inner, f)
            }
        }
        #[automatically_derived]
        impl ::ink::scale::EncodeLike for EventsRef {}
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for EventsRef {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "EventsRef",
                "inner",
                &&self.inner,
            )
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for EventsRef {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.inner, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for EventsRef {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for EventsRef {
        #[inline]
        fn eq(&self, other: &EventsRef) -> bool {
            self.inner == other.inner
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for EventsRef {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<
                <Events as ::ink::codegen::ContractCallBuilder>::Type,
            >;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for EventsRef {
        #[inline]
        fn clone(&self) -> EventsRef {
            EventsRef {
                inner: ::core::clone::Clone::clone(&self.inner),
            }
        }
    }
    const _: () = {
        impl ::ink::env::ContractReference for Events {
            type Type = EventsRef;
        }
        impl ::ink::env::call::ConstructorReturnType<EventsRef> for Events {
            type Output = EventsRef;
            type Error = ();
            fn ok(value: EventsRef) -> Self::Output {
                value
            }
        }
        impl<E> ::ink::env::call::ConstructorReturnType<EventsRef>
        for ::core::result::Result<Events, E>
        where
            E: ::ink::scale::Decode,
        {
            const IS_RESULT: bool = true;
            type Output = ::core::result::Result<EventsRef, E>;
            type Error = E;
            fn ok(value: EventsRef) -> Self::Output {
                ::core::result::Result::Ok(value)
            }
            fn err(err: Self::Error) -> ::core::option::Option<Self::Output> {
                ::core::option::Option::Some(::core::result::Result::Err(err))
            }
        }
        impl ::ink::env::ContractEnv for EventsRef {
            type Env = <Events as ::ink::env::ContractEnv>::Env;
        }
    };
    /// Implementing the trait from the `event_def_unused` crate includes all defined
    /// events there.
    impl event_def_unused::FlipperTrait for EventsRef {
        type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<
            Environment,
        > as event_def_unused::FlipperTrait>::__ink_TraitInfo;
        type flipOutput = <<Self::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder as event_def_unused::FlipperTrait>::flipOutput;
        #[inline]
        fn flip(&mut self) -> Self::flipOutput {
            <_ as event_def_unused::FlipperTrait>::flip(
                <_ as ::ink::codegen::TraitCallForwarderFor<
                    {
                        <<::ink::reflect::TraitDefinitionRegistry<
                            Environment,
                        > as event_def_unused::FlipperTrait>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
                    },
                >>::forward_mut(
                    <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self),
                ),
            )
        }
    }
    impl EventsRef {
        /// Creates a new events smart contract initialized with the given value.
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new(
            __ink_binding_0: bool,
        ) -> ::ink::env::call::CreateBuilder<
            Environment,
            Self,
            ::ink::env::call::utils::Set<::ink::env::call::LimitParamsV2>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::ArgumentList<
                        ::ink::env::call::utils::Argument<bool>,
                        ::ink::env::call::utils::EmptyArgumentList,
                    >,
                >,
            >,
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
        /// Flips the current value of the boolean.
        #[inline]
        pub fn flip_with_foreign_event(&mut self) {
            self.try_flip_with_foreign_event()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "flip_with_foreign_event", error
                        ),
                    );
                })
        }
        /// Flips the current value of the boolean.
        #[inline]
        pub fn try_flip_with_foreign_event(&mut self) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .flip_with_foreign_event()
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "flip_with_foreign_event", error
                        ),
                    );
                })
        }
        /// Flips the current value of the boolean.
        #[inline]
        pub fn flip_with_inline_event(&mut self) {
            self.try_flip_with_inline_event()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "flip_with_inline_event", error
                        ),
                    );
                })
        }
        /// Flips the current value of the boolean.
        #[inline]
        pub fn try_flip_with_inline_event(&mut self) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .flip_with_inline_event()
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "flip_with_inline_event", error
                        ),
                    );
                })
        }
        /// Flips the current value of the boolean.
        #[inline]
        pub fn flip_with_inline_custom_event(&mut self) {
            self.try_flip_with_inline_custom_event()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "flip_with_inline_custom_event", error
                        ),
                    );
                })
        }
        /// Flips the current value of the boolean.
        #[inline]
        pub fn try_flip_with_inline_custom_event(&mut self) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call_mut(self)
                .flip_with_inline_custom_event()
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "flip_with_inline_custom_event", error
                        ),
                    );
                })
        }
        /// Emit an event with a 32 byte topic.
        #[inline]
        pub fn emit_32_byte_topic_event(&self, maybe_hash: Option<[u8; 32]>) {
            self.try_emit_32_byte_topic_event(maybe_hash)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "emit_32_byte_topic_event", error
                        ),
                    );
                })
        }
        /// Emit an event with a 32 byte topic.
        #[inline]
        pub fn try_emit_32_byte_topic_event(
            &self,
            maybe_hash: Option<[u8; 32]>,
        ) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .emit_32_byte_topic_event(maybe_hash)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "emit_32_byte_topic_event", error
                        ),
                    );
                })
        }
        /// Emit an event from a different crate.
        #[inline]
        pub fn emit_event_from_a_different_crate(&self, maybe_hash: Option<[u8; 32]>) {
            self.try_emit_event_from_a_different_crate(maybe_hash)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "emit_event_from_a_different_crate", error
                        ),
                    );
                })
        }
        /// Emit an event from a different crate.
        #[inline]
        pub fn try_emit_event_from_a_different_crate(
            &self,
            maybe_hash: Option<[u8; 32]>,
        ) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .emit_event_from_a_different_crate(maybe_hash)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "emit_event_from_a_different_crate", error
                        ),
                    );
                })
        }
        /// Emit a inline and standalone anonymous events
        #[inline]
        pub fn emit_anonymous_events(&self, topic: [u8; 32]) {
            self.try_emit_anonymous_events(topic)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "emit_anonymous_events", error
                        ),
                    );
                })
        }
        /// Emit a inline and standalone anonymous events
        #[inline]
        pub fn try_emit_anonymous_events(
            &self,
            topic: [u8; 32],
        ) -> ::ink::MessageResult<()> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .emit_anonymous_events(topic)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}", "Events",
                            "emit_anonymous_events", error
                        ),
                    );
                })
        }
    }
    const _: () = {
        impl ::ink::codegen::TraitCallBuilder for EventsRef {
            type Builder = <Events as ::ink::codegen::ContractCallBuilder>::Type;
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
    impl ::ink::env::call::FromAddr for EventsRef {
        #[inline]
        fn from_addr(addr: ::ink::H160) -> Self {
            Self {
                inner: <<Events as ::ink::codegen::ContractCallBuilder>::Type as ::ink::env::call::FromAddr>::from_addr(
                    addr,
                ),
            }
        }
    }
    impl ::ink::ToAddr for EventsRef {
        #[inline]
        fn to_addr(&self) -> ::ink::H160 {
            <<Events as ::ink::codegen::ContractCallBuilder>::Type as ::ink::ToAddr>::to_addr(
                &self.inner,
            )
        }
    }
    impl ::core::convert::AsRef<::ink::H160> for EventsRef {
        fn as_ref(&self) -> &::ink::H160 {
            <_ as ::core::convert::AsRef<::ink::H160>>::as_ref(&self.inner)
        }
    }
    impl ::core::convert::AsMut<::ink::H160> for EventsRef {
        fn as_mut(&mut self) -> &mut ::ink::H160 {
            <_ as ::core::convert::AsMut<::ink::H160>>::as_mut(&mut self.inner)
        }
    }
}
