#![feature(prelude_import)]
#![feature(proc_macro_hygiene)]
#![allow(unused_imports)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
use ink_core::storage;
use ink_lang2 as ink;
mod erc20 {
    use super::*;
    type Env = ink_core::env2::EnvImpl<ink_core::env2::DefaultSrmlTypes>;
    type AccountId = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::AccountId;
    type Balance = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::Balance;
    type Hash = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::Hash;
    type Moment = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::Moment;
    type BlockNumber = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::BlockNumber;
    mod __ink_private {
        use super::*;
        #[cfg(not(feature = "ink-as-dependency"))]
        pub use super::{Approval, Transfer};
        #[doc(hidden)]
        #[cfg(not(feature = "ink-as-dependency"))]
        mod __ink_storage {
            use super::*;
            pub type UsedEnv = ink_core::env2::EnvAccess<Env>;
            impl ink_lang2::AccessEnv<Env> for StorageAndEnv {
                fn access_env(&mut self) -> &mut ink_core::env2::EnvAccess<Env> {
                    &mut self.__env
                }
            }
            impl<'a> ink_core::env2::AccessEnv for &'a StorageAndEnv {
                type Target = <&'a UsedEnv as ink_core::env2::AccessEnv>::Target;
                fn env(self) -> Self::Target {
                    ink_core::env2::AccessEnv::env(&self.__env)
                }
            }
            impl<'a> ink_core::env2::AccessEnv for &'a mut StorageAndEnv {
                type Target = <&'a mut UsedEnv as ink_core::env2::AccessEnv>::Target;
                fn env(self) -> Self::Target {
                    ink_core::env2::AccessEnv::env(&mut self.__env)
                }
            }
            impl ink_core::storage::alloc::AllocateUsing for Storage {
                unsafe fn allocate_using<A>(alloc: &mut A) -> Self
                where
                    A: ink_core::storage::alloc::Allocate,
                {
                    Self {
                        total_supply: ink_core::storage::alloc::AllocateUsing::allocate_using(
                            alloc,
                        ),
                        balances: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                        allowances: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                    }
                }
            }
            impl ink_core::storage::Flush for Storage {
                fn flush(&mut self) {
                    ink_core::storage::Flush::flush(&mut self.total_supply);
                    ink_core::storage::Flush::flush(&mut self.balances);
                    ink_core::storage::Flush::flush(&mut self.allowances);
                }
            }
            impl ink_core::storage::alloc::Initialize for Storage {
                type Args = ();
                fn default_value() -> Option<Self::Args> {
                    Some(())
                }
                fn initialize(&mut self, _args: Self::Args) {
                    self.total_supply.try_default_initialize();
                    self.balances.try_default_initialize();
                    self.allowances.try_default_initialize();
                }
            }
            pub struct Storage {
                pub total_supply: storage::Value<Balance>,
                pub balances: storage::HashMap<AccountId, Balance>,
                pub allowances: storage::HashMap<(AccountId, AccountId), Balance>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for Storage {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        Storage {
                            total_supply: ref __self_0_0,
                            balances: ref __self_0_1,
                            allowances: ref __self_0_2,
                        } => {
                            let mut debug_trait_builder = f.debug_struct("Storage");
                            let _ = debug_trait_builder.field("total_supply", &&(*__self_0_0));
                            let _ = debug_trait_builder.field("balances", &&(*__self_0_1));
                            let _ = debug_trait_builder.field("allowances", &&(*__self_0_2));
                            debug_trait_builder.finish()
                        }
                    }
                }
            }
            pub struct StorageAndEnv {
                __storage: Storage,
                __env: UsedEnv,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for StorageAndEnv {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        StorageAndEnv {
                            __storage: ref __self_0_0,
                            __env: ref __self_0_1,
                        } => {
                            let mut debug_trait_builder = f.debug_struct("StorageAndEnv");
                            let _ = debug_trait_builder.field("__storage", &&(*__self_0_0));
                            let _ = debug_trait_builder.field("__env", &&(*__self_0_1));
                            debug_trait_builder.finish()
                        }
                    }
                }
            }
            impl core::ops::Deref for StorageAndEnv {
                type Target = Storage;
                fn deref(&self) -> &Self::Target {
                    &self.__storage
                }
            }
            impl core::ops::DerefMut for StorageAndEnv {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.__storage
                }
            }
            impl ink_core::storage::alloc::AllocateUsing for StorageAndEnv {
                unsafe fn allocate_using<A>(alloc: &mut A) -> Self
                where
                    A: ink_core::storage::alloc::Allocate,
                {
                    Self {
                        __storage: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                        __env: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                    }
                }
            }
            impl ink_core::storage::Flush for StorageAndEnv {
                fn flush(&mut self) {
                    ink_core::storage::Flush::flush(&mut self.__storage);
                    ink_core::storage::Flush::flush(&mut self.__env);
                }
            }
            impl ink_core::storage::alloc::Initialize for StorageAndEnv {
                type Args = ();
                fn default_value() -> Option<Self::Args> {
                    Some(())
                }
                fn initialize(&mut self, _args: Self::Args) {
                    ink_core::storage::alloc::Initialize::try_default_initialize(
                        &mut self.__storage,
                    );
                    ink_core::storage::alloc::Initialize::try_default_initialize(&mut self.__env);
                }
            }
            impl ink_lang2::Storage for StorageAndEnv {}
        }
        #[cfg(not(feature = "ink-as-dependency"))]
        pub use __ink_storage::StorageAndEnv;
        #[cfg(not(feature = "ink-as-dependency"))]
        const _: () = {
            use __ink_private::EmitEvent as _;
            #[allow(unused_imports)]
            use ink_core::env2::AccessEnv as _;
            impl StorageAndEnv {
                pub fn new(&mut self, initial_supply: Balance) {
                    let caller = self.env().caller();
                    self.total_supply.set(initial_supply);
                    self.balances.insert(caller, initial_supply);
                    self.env().emit_event(Transfer {
                        from: None,
                        to: Some(caller),
                        value: initial_supply,
                    });
                }
                pub fn total_supply(&self) -> Balance {
                    *self.total_supply
                }
                pub fn balance_of(&self, owner: AccountId) -> Balance {
                    self.balance_of_or_zero(&owner)
                }
                pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
                    self.allowance_of_or_zero(&owner, &spender)
                }
                pub fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
                    let from = self.env().caller();
                    self.transfer_from_to(from, to, value)
                }
                pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
                    let owner = self.env().caller();
                    self.allowances.insert((owner, spender), value);
                    self.env().emit_event(Approval {
                        owner,
                        spender,
                        value,
                    });
                    true
                }
                pub fn transfer_from(
                    &mut self,
                    from: AccountId,
                    to: AccountId,
                    value: Balance,
                ) -> bool {
                    let caller = self.env().caller();
                    let allowance = self.allowance_of_or_zero(&from, &caller);
                    if allowance < value {
                        return false;
                    }
                    self.allowances.insert((from, caller), allowance - value);
                    self.transfer_from_to(from, to, value)
                }
                fn transfer_from_to(
                    &mut self,
                    from: AccountId,
                    to: AccountId,
                    value: Balance,
                ) -> bool {
                    let from_balance = self.balance_of_or_zero(&from);
                    if from_balance < value {
                        return false;
                    }
                    let to_balance = self.balance_of_or_zero(&to);
                    self.balances.insert(from.clone(), from_balance - value);
                    self.balances.insert(to.clone(), to_balance + value);
                    self.env().emit_event(Transfer {
                        from: Some(from),
                        to: Some(to),
                        value,
                    });
                    true
                }
                fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
                    *self.balances.get(owner).unwrap_or(&0)
                }
                fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
                    *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
                }
            }
        };
        #[cfg(not(feature = "ink-as-dependency"))]
        mod __ink_events {
            use super::{AccountId, Balance, BlockNumber, Env, Hash, Moment};
            #[cfg(not(feature = "ink-as-dependency"))]
            pub use super::{Approval, Transfer};
            impl ink_core::env2::Topics<Env> for Transfer {
                fn topics(&self) -> &'static [Hash] {
                    &[]
                }
            }
            impl ink_core::env2::Topics<Env> for Approval {
                fn topics(&self) -> &'static [Hash] {
                    &[]
                }
            }
            pub enum Event {
                Transfer(Transfer),
                Approval(Approval),
            }
            const _: () = {
                #[allow(unknown_lints)]
                #[allow(rust_2018_idioms)]
                extern crate scale as _parity_scale_codec;
                impl _parity_scale_codec::Encode for Event {
                    fn encode_to<EncOut: _parity_scale_codec::Output>(&self, dest: &mut EncOut) {
                        match *self {
                            Event::Transfer(ref aa) => {
                                dest.push_byte(0usize as u8);
                                dest.push(aa);
                            }
                            Event::Approval(ref aa) => {
                                dest.push_byte(1usize as u8);
                                dest.push(aa);
                            }
                            _ => (),
                        }
                    }
                }
                impl _parity_scale_codec::EncodeLike for Event {}
            };
            impl From<Transfer> for Event {
                fn from(event: Transfer) -> Self {
                    Event::Transfer(event)
                }
            }
            impl From<Approval> for Event {
                fn from(event: Approval) -> Self {
                    Event::Approval(event)
                }
            }
            impl ink_core::env2::Topics<Env> for Event {
                fn topics(&self) -> &'static [Hash] {
                    match self {
                        Event::Transfer(event) => event.topics(),
                        Event::Approval(event) => event.topics(),
                    }
                }
            }
            pub trait EmitEvent {
                fn emit_event<E>(self, event: E)
                where
                    E: Into<Event>;
            }
            impl<'a> EmitEvent for &'a mut ink_core::env2::EnvAccessMut<Env> {
                fn emit_event<E>(self, event: E)
                where
                    E: Into<Event>,
                {
                    ink_core::env2::EmitEvent::emit_event(self, event.into())
                }
            }
        }
        #[cfg(not(feature = "ink-as-dependency"))]
        pub use __ink_events::{EmitEvent, Event};
    }
    #[cfg(not(all(test, feature = "test-env")))]
    #[cfg(not(feature = "ink-as-dependency"))]
    pub type Erc20 = self::__ink_private::StorageAndEnv;
    #[cfg(not(feature = "ink-as-dependency"))]
    pub struct Transfer {
        pub from: Option<AccountId>,
        pub to: Option<AccountId>,
        pub value: Balance,
    }
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate scale as _parity_scale_codec;
        impl _parity_scale_codec::Encode for Transfer {
            fn encode_to<EncOut: _parity_scale_codec::Output>(&self, dest: &mut EncOut) {
                dest.push(&self.from);
                dest.push(&self.to);
                dest.push(&self.value);
            }
        }
        impl _parity_scale_codec::EncodeLike for Transfer {}
    };
    #[cfg(not(feature = "ink-as-dependency"))]
    pub struct Approval {
        pub owner: AccountId,
        pub spender: AccountId,
        pub value: Balance,
    }
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate scale as _parity_scale_codec;
        impl _parity_scale_codec::Encode for Approval {
            fn encode_to<EncOut: _parity_scale_codec::Output>(&self, dest: &mut EncOut) {
                dest.push(&self.owner);
                dest.push(&self.spender);
                dest.push(&self.value);
            }
        }
        impl _parity_scale_codec::EncodeLike for Approval {}
    };
}
