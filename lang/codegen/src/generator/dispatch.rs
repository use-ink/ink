// Copyright 2018-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    generator,
    GenerateCode,
    GenerateCodeUsing as _,
};
use derive_more::From;
use ir::Callable as _;
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::quote;

/// Generates code for the message and constructor dispatcher.
///
/// This code efficiently selects the dispatched ink! constructor or message
/// by inspecting the first four bytes (selector) of the given input bytes.
///
/// As this happens on every contract execution this code must be highly optimized.
/// For that purpose a so-called dispatch enum is being generated that has a
/// specialized `scale::Decode` implementation taking the first four bytes of
/// the input stream in order to identify the enum variant that it is going to
/// produce out of the rest of the input buffer.
///
/// The rest of the input buffer is then automatically decoded directly into the
/// expected input types of the respective ink! constructor or message.
#[derive(From)]
pub struct Dispatch<'a> {
    contract: &'a ir::Contract,
}

impl AsRef<ir::Contract> for Dispatch<'_> {
    fn as_ref(&self) -> &ir::Contract {
        self.contract
    }
}

impl GenerateCode for Dispatch<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let no_cross_calling_cfg =
            self.generate_code_using::<generator::CrossCallingConflictCfg>();
        let entry_points = self.generate_entry_points();
        let dispatch_using_mode = self.generate_dispatch_using_mode();
        let message_dispatch_enum = self.generate_message_dispatch_enum();
        let constructor_dispatch_enum = self.generate_constructor_dispatch_enum();
        quote! {
            // We do not generate contract dispatch code while the contract
            // is being tested or the contract is a dependency of another
            // since both resulting compilations do not require dispatching.
            #[cfg(not(test))]
            #no_cross_calling_cfg
            const _: () = {
                #entry_points
                #dispatch_using_mode
                #message_dispatch_enum
                #constructor_dispatch_enum
            };
        }
    }
}

impl Dispatch<'_> {
    /// Generates the static ink! contract entry points.
    ///
    /// # Note
    ///
    /// Those are expected to exist by the smart contracts host module.
    /// They guide the dispatch, set-up and tear-down of a smart contract.
    fn generate_entry_points(&self) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();
        quote! {
            #[cfg(not(test))]
            #[no_mangle]
            fn deploy() -> u32 {
                ::ink_lang::DispatchRetCode::from(
                    <#storage_ident as ::ink_lang::DispatchUsingMode>::dispatch_using_mode(
                        ::ink_lang::DispatchMode::Instantiate,
                    ),
                )
                .to_u32()
            }

            #[cfg(not(test))]
            #[no_mangle]
            fn call() -> u32 {
                ::ink_lang::DispatchRetCode::from(
                    <#storage_ident as ::ink_lang::DispatchUsingMode>::dispatch_using_mode(
                        ::ink_lang::DispatchMode::Call,
                    ),
                )
                .to_u32()
            }
        }
    }

    /// Generates the `DispatchUsingMode` trait implementation to guide contract dispatch.
    fn generate_dispatch_using_mode(&self) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();
        quote! {
            impl ::ink_lang::DispatchUsingMode for #storage_ident {
                #[allow(unused_parens)]
                fn dispatch_using_mode(
                    mode: ::ink_lang::DispatchMode
                ) -> core::result::Result<(), ::ink_lang::DispatchError> {
                    match mode {
                        ::ink_lang::DispatchMode::Instantiate => {
                            <<#storage_ident as ::ink_lang::ConstructorDispatcher>::Type as ::ink_lang::Execute>::execute(
                                ::ink_core::env::decode_input::<<#storage_ident as ::ink_lang::ConstructorDispatcher>::Type>()
                                    .map_err(|_| ::ink_lang::DispatchError::CouldNotReadInput)?
                            )
                        }
                        ::ink_lang::DispatchMode::Call => {
                            <<#storage_ident as ::ink_lang::MessageDispatcher>::Type as ::ink_lang::Execute>::execute(
                                ::ink_core::env::decode_input::<<#storage_ident as ::ink_lang::MessageDispatcher>::Type>()
                                    .map_err(|_| ::ink_lang::DispatchError::CouldNotReadInput)?
                            )
                        }
                    }
                }
            }
        }
    }

    /// Generates variant identifiers for the generated dispatch enum.
    ///
    /// Since we want to avoid generating random names we generate identifiers
    /// in terms of the selectors of the associated ink! messages or constructors.
    ///
    /// ## Example
    ///
    /// Given prefix of `"Message"` and selector with bytes `0xDEADBEEF` we
    /// generate the following idenfitier: `__ink_Message_0xDEADBEEF`
    ///
    /// This way it is clear that this is an ink! generated identifier and even
    /// encodes the unique selector bytes to make the identifier unique.
    fn generate_dispatch_variant_ident<C>(
        &self,
        cws: ir::CallableWithSelector<'_, C>,
    ) -> Ident
    where
        C: ir::Callable,
    {
        let selector_bytes = cws.composed_selector().as_bytes().to_owned();
        let prefix = match cws.callable().kind() {
            ir::CallableKind::Message => "Message",
            ir::CallableKind::Constructor => "Constructor",
        };
        quote::format_ident!(
            "__{}_0x{:02X}{:02X}{:02X}{:02X}",
            prefix,
            selector_bytes[0],
            selector_bytes[1],
            selector_bytes[2],
            selector_bytes[3]
        )
    }

    /// Generates one match arm of the dispatch `scale::Decode` implementation.
    ///
    /// # Note
    ///
    /// There is one match arm per ink! constructor or message for the dispatch
    /// `scale::Decode` implementation.
    fn generate_dispatch_variant_decode<C>(
        &self,
        cws: ir::CallableWithSelector<'_, C>,
    ) -> TokenStream2
    where
        C: ir::Callable,
    {
        let selector_bytes = cws.composed_selector().as_bytes().to_owned();
        let variant_ident = self.generate_dispatch_variant_ident(cws);
        let variant_types = cws.callable().inputs().map(|arg| &arg.ty);
        quote! {
            [ #( #selector_bytes ),* ] => {
                Ok(Self::#variant_ident(
                    #(
                        <#variant_types as ::scale::Decode>::decode(input)?
                    ),*
                ))
            }
        }
    }

    /// Generates one match arm of the dispatch variant enum.
    ///
    /// # Note
    ///
    /// There is one match arm per ink! constructor or message for the dispatch
    /// `scale::Decode` implementation.
    fn generate_dispatch_variant_arm<C>(
        &self,
        cws: ir::CallableWithSelector<'_, C>,
    ) -> TokenStream2
    where
        C: ir::Callable,
    {
        let input_types = cws.callable().inputs().map(|arg| &arg.ty);
        let variant_ident = self.generate_dispatch_variant_ident(cws);
        quote! {
            #variant_ident(#(#input_types),*)
        }
    }

    /// Generates one match arm of the dispatch message for the `execute` implementation.
    ///
    /// # Note
    ///
    /// This is basically the code per ink! message that is going to be executed after
    /// the dispatch has already taken place.
    fn generate_dispatch_execute_message_arm(
        &self,
        cws: ir::CallableWithSelector<'_, ir::Message>,
    ) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();
        let ident = self.generate_dispatch_variant_ident(cws);
        let message = cws.callable();
        let arg_pats = message.inputs().map(|arg| &arg.pat).collect::<Vec<_>>();
        let arg_inputs = if arg_pats.len() == 1 {
            quote! { #(#arg_pats),* }
        } else {
            quote! { ( #(#arg_pats),* ) }
        };
        let (mut_mod, msg_trait, exec_fn) = match message.receiver() {
            ir::Receiver::RefMut => {
                (
                    Some(quote! { mut }),
                    quote! { MessageMut },
                    quote! { execute_message_mut },
                )
            }
            ir::Receiver::Ref => {
                (None, quote! { MessageRef }, quote! { execute_message })
            }
        };
        let selector_id = cws.composed_selector().unique_id();
        quote! {
            Self::#ident(#(#arg_pats),*) => {
                ::ink_lang::#exec_fn::<Msg<[(); #selector_id]>, _>(move |state: &#mut_mod #storage_ident| {
                    <Msg<[(); #selector_id]> as ::ink_lang::#msg_trait>::CALLABLE(
                        state, #arg_inputs
                    )
                })
            }
        }
    }

    /// Returns an iterator over all ink! messages of the ink! contract.
    fn contract_messages(
        &self,
    ) -> impl Iterator<Item = ir::CallableWithSelector<ir::Message>> {
        self.contract
            .module()
            .impls()
            .map(|impl_item| impl_item.iter_messages())
            .flatten()
    }

    /// Generates the entire dispatch variant enum for all ink! messages.
    fn generate_message_dispatch_enum(&self) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();
        let message_variants = self
            .contract_messages()
            .map(|message| self.generate_dispatch_variant_arm(message));
        let decode_message = self
            .contract_messages()
            .map(|message| self.generate_dispatch_variant_decode(message));
        let execute_variants = self
            .contract_messages()
            .map(|message| self.generate_dispatch_execute_message_arm(message));
        quote! {
            const _: () = {
                pub enum MessageDispatchEnum {
                    #( #message_variants ),*
                }

                impl ::ink_lang::MessageDispatcher for #storage_ident {
                    type Type = MessageDispatchEnum;
                }

                impl ::scale::Decode for MessageDispatchEnum {
                    fn decode<I: ::scale::Input>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error> {
                        match <[u8; 4] as ::scale::Decode>::decode(input)? {
                            #( #decode_message )*
                            _invalid => Err(::scale::Error::from("encountered unknown ink! message selector"))
                        }
                    }
                }

                impl ::ink_lang::Execute for MessageDispatchEnum {
                    fn execute(self) -> ::core::result::Result<(), ::ink_lang::DispatchError> {
                        match self {
                            #( #execute_variants )*
                        }
                    }
                }
            };
        }
    }

    /// Generates one match arm of the dispatch constructor for the `execute` implementation.
    ///
    /// # Note
    ///
    /// This is basically the code per ink! constructor that is going to be executed after
    /// the dispatch has already taken place.
    fn generate_dispatch_execute_constructor_arm(
        &self,
        cws: ir::CallableWithSelector<'_, ir::Constructor>,
    ) -> TokenStream2 {
        let ident = self.generate_dispatch_variant_ident(cws);
        let constructor = cws.callable();
        let arg_pats = constructor.inputs().map(|arg| &arg.pat).collect::<Vec<_>>();
        let arg_inputs = if arg_pats.len() == 1 {
            quote! { #(#arg_pats),* }
        } else {
            quote! { ( #(#arg_pats),* ) }
        };
        let selector_id = cws.composed_selector().unique_id();
        quote! {
            Self::#ident(#(#arg_pats),*) => {
                ::ink_lang::execute_constructor::<Constr<[(); #selector_id]>, _>(move || {
                    <Constr<[(); #selector_id]> as ::ink_lang::Constructor>::CALLABLE(
                        #arg_inputs
                    )
                })
            }
        }
    }

    /// Returns an iterator over all ink! constructors of the ink! contract.
    fn contract_constructors(
        &self,
    ) -> impl Iterator<Item = ir::CallableWithSelector<ir::Constructor>> {
        self.contract
            .module()
            .impls()
            .map(|impl_item| impl_item.iter_constructors())
            .flatten()
    }

    /// Generates the entire dispatch variant enum for all ink! messages.
    fn generate_constructor_dispatch_enum(&self) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();
        let message_variants = self
            .contract_messages()
            .map(|message| self.generate_dispatch_variant_arm(message));
        let decode_message = self
            .contract_messages()
            .map(|message| self.generate_dispatch_variant_decode(message));
        let execute_variants = self
            .contract_constructors()
            .map(|cws| self.generate_dispatch_execute_constructor_arm(cws));
        quote! {
            const _: () = {
                pub enum ConstructorDispatchEnum {
                    #( #message_variants ),*
                }

                impl ::ink_lang::ConstructorDispatcher for #storage_ident {
                    type Type = ConstructorDispatchEnum;
                }

                impl ::scale::Decode for ConstructorDispatchEnum {
                    fn decode<I: ::scale::Input>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error> {
                        match <[u8; 4] as ::scale::Decode>::decode(input)? {
                            #( #decode_message )*
                            _invalid => Err(::scale::Error::from("encountered unknown ink! constructor selector"))
                        }
                    }
                }

                impl ::ink_lang::Execute for ConstructorDispatchEnum {
                    fn execute(self) -> ::core::result::Result<(), ::ink_lang::DispatchError> {
                        match self {
                            #( #execute_variants )*
                        }
                    }
                }
            };
        }
    }
}
