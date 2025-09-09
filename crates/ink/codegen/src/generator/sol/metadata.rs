// Copyright (C) ink! contributors.
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
use ir::{
    Callable as _,
    InputsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Pat;

use super::utils::{
    extract_docs,
    sol_return_type,
    sol_type,
};
use crate::GenerateCode;

/// Generates code for generating Solidity ABI compatibility metadata for the contract.
#[derive(From)]
pub struct SolidityMetadata<'a> {
    /// The contract to generate code for.
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(SolidityMetadata);

impl GenerateCode for SolidityMetadata<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let ident = self.contract.module().storage().ident();
        let name = ident.to_string();
        let ctors = self.constructors();
        let msgs = self.messages();
        let docs = extract_docs(self.contract.module().attrs());

        quote! {
            #[cfg(feature = "std")]
            #[cfg(not(feature = "ink-as-dependency"))]
            #[cfg(any(ink_abi = "sol", ink_abi = "all"))]
            const _: () = {
                #[unsafe(no_mangle)]
                pub fn __ink_generate_solidity_metadata() -> ::ink::metadata::sol::ContractMetadata  {
                    ::ink::metadata::sol::ContractMetadata {
                        name: #name.into(),
                        constructors: vec![ #( #ctors ),* ],
                        functions: vec![ #( #msgs ),* ],
                        events: ::ink::collect_events_sol(),
                        errors: ::ink::collect_errors_sol(),
                        docs: #docs.into(),
                    }
                }
            };
        }
    }
}

impl SolidityMetadata<'_> {
    /// Generates Solidity ABI compatible metadata for all ink! constructors.
    fn constructors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .map(|ctor| {
                let name = ctor
                    .name()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| ctor.ident().to_string());
                let inputs = params_info(ctor.inputs());
                let is_payable = ctor.is_payable();
                let is_default = ctor.is_default();
                let docs = extract_docs(ctor.attrs());

                quote! {
                    ::ink::metadata::sol::ConstructorMetadata {
                        name: #name.into(),
                        inputs: vec![ #( #inputs ),* ],
                        is_payable: #is_payable,
                        is_default: #is_default,
                        docs: #docs.into(),
                    }
                }
            })
    }

    /// Generates Solidity ABI compatible metadata for all ink! messages.
    fn messages(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_messages())
            .map(|msg| {
                let name = msg
                    .name()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| msg.ident().to_string());
                let inputs = params_info(msg.inputs());
                let output = msg
                    .output()
                    .map(|ty| {
                        let sol_ty = sol_return_type(ty);
                        quote! { ::core::option::Option::Some(#sol_ty.into()) }
                    })
                    .unwrap_or_else(|| {
                        quote! { ::core::option::Option::None }
                    });
                let mutates = msg.receiver().is_ref_mut();
                let is_payable = msg.is_payable();
                let is_default = msg.is_default();
                let docs = extract_docs(msg.attrs());

                quote! {
                    ::ink::metadata::sol::FunctionMetadata {
                        name: #name.into(),
                        inputs: vec![ #( #inputs ),* ],
                        output: #output,
                        mutates: #mutates,
                        is_payable: #is_payable,
                        is_default: #is_default,
                        docs: #docs.into(),
                    }
                }
            })
    }
}

/// Returns the Solidity ABI compatible parameter type and name for the given inputs.
fn params_info(inputs: InputsIter<'_>) -> impl Iterator<Item = TokenStream2> + '_ {
    inputs.map(|input| {
        let ty = &*input.ty;
        let sol_ty = sol_type(ty);
        let ident = match &*input.pat {
            Pat::Ident(ident) => &ident.ident,
            _ => unreachable!("Expected an input identifier"),
        };
        let name = ident.to_string();
        quote! {
            ::ink::metadata::sol::ParamMetadata {
                name: #name.into(),
                ty: #sol_ty.into(),
            }
        }
    })
}
