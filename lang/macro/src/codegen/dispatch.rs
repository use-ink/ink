// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    punctuated::Punctuated,
    Token,
};

use crate::{
    codegen::{
        cross_calling::CrossCallingConflictCfg,
        GenerateCode,
        GenerateCodeUsing,
    },
    ir,
};

/// Generates code for the dispatch parts that dispatch constructors
/// and messages from the input and also handle the returning of data.
#[derive(From)]
pub struct Dispatch<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}

impl<'a> GenerateCodeUsing for Dispatch<'a> {
    fn contract(&self) -> &ir::Contract {
        self.contract
    }
}

impl GenerateCode for Dispatch<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let conflic_depedency_cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        let message_trait_impls = self.generate_message_trait_impls();
        let message_namespaces = self.generate_message_namespaces();
        let dispatch_using_mode = self.generate_dispatch_using_mode();
        let entry_points = self.generate_entry_points();

        quote! {
            // We do not generate contract dispatch code
            // while the contract is being tested or the
            // `test-env` has been enabled since both resulting
            // compilations do not require dispatching.
            #[cfg(not(any(test, feature = "test-env")))]
            #conflic_depedency_cfg
            const _: () = {
                #message_namespaces
                #message_trait_impls
                #dispatch_using_mode
                #entry_points
            };
        }
    }
}

impl Dispatch<'_> {
    fn generate_trait_impls_for_message(&self, function: &ir::Function) -> TokenStream2 {
        if !(function.is_constructor() || function.is_message()) {
            return quote! {}
        }
        let span = function.span();
        let selector = function
            .selector()
            .expect("this is either a message or constructor at this point; qed");
        let (selector_bytes, selector_id) = (selector.as_bytes(), selector.unique_id());
        let sig = &function.sig;
        let inputs = sig.inputs().map(|ident_type| &ident_type.ty);
        let inputs_punct = inputs.collect::<Punctuated<_, Token![,]>>();
        let output = &sig.output;
        let output_type = match output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };
        let is_mut = sig.is_mut().unwrap_or(true);
        let is_constructor = function.is_constructor();
        let state_ident = &self.contract.storage.ident;
        let fn_ident = &function.sig.ident;

        use syn::spanned::Spanned as _;

        let namespace = match function.kind() {
            ir::FunctionKind::Constructor(_) => quote! { Constr },
            ir::FunctionKind::Message(_) => quote! { Msg },
            ir::FunctionKind::Method => panic!("ICE: can't match a method at this point"),
        };
        let inputs = if inputs_punct.len() != 1 {
            quote! { ( #inputs_punct )}
        } else {
            quote! { #inputs_punct }
        };
        let fn_input = quote_spanned!(sig.inputs.span() =>
            impl ::ink_lang::v2::FnInput for #namespace<[(); #selector_id]> {
                type Input = #inputs;
            }
        );
        let fn_output2 = if !is_constructor {
            quote_spanned!(sig.output.span() =>
                impl ::ink_lang::v2::FnOutput for #namespace<[(); #selector_id]> {
                    #[allow(unused_parens)]
                    type Output = #output_type;
                }
            )
        } else {
            quote! {}
        };
        let fn_selector = quote_spanned!(span =>
            impl ::ink_lang::v2::FnSelector for #namespace<[(); #selector_id]> {
                const SELECTOR: ink_core::env::call::Selector = ::ink_core::env::call::Selector::new([
                    #( #selector_bytes ),*
                ]);
            }
        );
        let fn_state = quote_spanned!(span =>
            impl ::ink_lang::v2::FnState for #namespace<[(); #selector_id]> {
                type State = #state_ident;
            }
        );
        let input_idents = sig
            .inputs()
            .map(|ident_type| &ident_type.ident)
            .collect::<Punctuated<_, Token![,]>>();
        let input_params = if input_idents.len() >= 2 {
            quote! { (#input_idents) }
        } else if input_idents.len() == 1 {
            quote! { #input_idents }
        } else {
            quote! { _ }
        };
        let input_forward = quote! { #input_idents };
        let message2_impl = if is_constructor {
            quote_spanned!(span =>
                impl ::ink_lang::v2::Constructor for #namespace<[(); #selector_id]> {
                    const CALLABLE: fn(
                        <Self as ::ink_lang::v2::FnInput>::Input
                    ) -> <Self as ::ink_lang::v2::FnState>::State = |#input_params| #state_ident::#fn_ident(#input_forward);
                }
            )
        } else if is_mut {
            quote_spanned!(span =>
                impl ::ink_lang::v2::MessageMut for #namespace<[(); #selector_id]> {
                    const CALLABLE: fn(
                        &mut <Self as ::ink_lang::v2::FnState>::State,
                        <Self as ::ink_lang::v2::FnInput>::Input
                    ) -> <Self as ::ink_lang::v2::FnOutput>::Output = |state, #input_params| #state_ident::#fn_ident(state, #input_forward);
                }
            )
        } else {
            quote_spanned!(span =>
                impl ::ink_lang::v2::MessageRef for #namespace<[(); #selector_id]> {
                    const CALLABLE: fn(
                        &<Self as ::ink_lang::v2::FnState>::State,
                        <Self as ::ink_lang::v2::FnInput>::Input
                    ) -> <Self as ::ink_lang::v2::FnOutput>::Output = |state, #input_params| #state_ident::#fn_ident(state, #input_forward);
                }
            )
        };

        quote_spanned!(span =>
            #fn_input
            #fn_output2
            #fn_selector
            #fn_state
            #message2_impl
        )
    }

    fn generate_message_trait_impls(&self) -> TokenStream2 {
        let fns = self
            .contract
            .functions
            .iter()
            .map(|fun| self.generate_trait_impls_for_message(fun));
        quote! {
            #( #fns )*
        }
    }

    fn generate_message_namespaces(&self) -> TokenStream2 {
        quote! {
            // Namespace for messages.
            //
            // # Note
            //
            // The `S` parameter is going to refer to array types `[(); N]`
            // where `N` is the unique identifier of the associated message
            // selector.
            pub struct Msg<S> {
                // We need to wrap inner because of Rust's orphan rules.
                marker: core::marker::PhantomData<fn() -> S>,
            }

            // Namespace for constructors.
            //
            // # Note
            //
            // The `S` parameter is going to refer to array types `[(); N]`
            // where `N` is the unique identifier of the associated constructor
            // selector.
            pub struct Constr<S> {
                // We need to wrap inner because of Rust's orphan rules.
                marker: core::marker::PhantomData<fn() -> S>,
            }
        }
    }

    fn generate_dispatch_using_mode_fragment(
        &self,
        function: &ir::Function,
    ) -> TokenStream2 {
        if !(function.is_constructor() || function.is_message()) {
            return quote! {}
        }
        let selector = function
            .selector()
            .expect("this is either a message or constructor at this point; qed");
        let selector_id = selector.unique_id();
        let sig = &function.sig;
        let builder_name = if function.is_constructor() {
            quote! { register_constructor }
        } else if sig.is_mut().expect("must be a message if not constructor") {
            quote! { register_message_mut }
        } else {
            quote! { register_message }
        };
        let namespace = match function.kind() {
            ir::FunctionKind::Constructor(_) => quote! { Constr },
            ir::FunctionKind::Message(_) => quote! { Msg },
            ir::FunctionKind::Method => panic!("ICE: can't match a method at this point"),
        };
        quote! {
            .#builder_name::<#namespace<[(); #selector_id]>>()
        }
    }

    fn generate_dispatch_using_mode(&self) -> TokenStream2 {
        let fragments = self
            .contract
            .functions
            .iter()
            .map(|fun| self.generate_dispatch_using_mode_fragment(fun));

        quote! {
            impl ink_lang::DispatchUsingMode for Storage {
                #[allow(unused_parens)]
                fn dispatch_using_mode(
                    mode: ink_lang::DispatchMode
                ) -> core::result::Result<(), ::ink_lang::DispatchError> {
                    let call_data =
                        ::ink_core::env::input().map_err(|_| ::ink_lang::DispatchError::CouldNotReadInput)?;
                    let contract = ::ink_lang::v2::Contract::build()
                        #(
                            #fragments
                        )*
                        .finalize();
                    match mode {
                        ::ink_lang::DispatchMode::Instantiate => contract.on_instantiate(&call_data),
                        ::ink_lang::DispatchMode::Call => contract.on_call(&call_data),
                    }
                }
            }
        }
    }

    fn generate_entry_points(&self) -> TokenStream2 {
        quote! {
            #[cfg(not(test))]
            #[no_mangle]
            fn deploy() -> u32 {
                ink_lang::DispatchRetCode::from(
                    <Storage as ink_lang::DispatchUsingMode>::dispatch_using_mode(
                        ink_lang::DispatchMode::Instantiate,
                    ),
                )
                .to_u32()
            }

            #[cfg(not(test))]
            #[no_mangle]
            fn call() -> u32 {
                ink_lang::DispatchRetCode::from(
                    <Storage as ink_lang::DispatchUsingMode>::dispatch_using_mode(
                        ink_lang::DispatchMode::Call,
                    ),
                )
                .to_u32()
            }
        }
    }
}
