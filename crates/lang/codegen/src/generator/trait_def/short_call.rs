// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

//! Generates code to implement an ink! trait definition for the concretized
//! type using the short-hand calling notation.

use super::TraitDefinition;
use heck::CamelCase as _;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

impl<'a> TraitDefinition<'a> {
    /// Generates the code to allow short-hand cross-chain contract calls for messages.
    ///
    /// Unlike the generated code for ink! trait constructors the generated code uses
    /// the long-hand calling versions under the hood.
    fn generate_trait_impl_block_message(
        &self,
        message: ir::InkTraitMessage,
        _selector: ir::Selector,
    ) -> TokenStream2 {
        let implementer_ident = self.concrete_implementer_ident();
        let span = message.span();
        let ident = message.ident();
        let output_ident = format_ident!("{}Out", ident.to_string().to_camel_case());
        let ident_str = ident.to_string();
        let trait_ident = self.trait_def.ident();
        let error_str = format!(
            "encountered error while calling <{} as {}>::{}",
            implementer_ident, trait_ident, ident_str
        );
        let inputs_sig = message.inputs();
        let inputs_params = message.inputs().map(|pat_type| &pat_type.pat);
        let output_ty = message
            .output()
            .cloned()
            .unwrap_or_else(|| syn::parse_quote! { () });
        let receiver = message.receiver();
        let forward_ident = match receiver {
            ir::Receiver::Ref => format_ident!("call"),
            ir::Receiver::RefMut => format_ident!("call_mut"),
        };
        let forward_trait = match receiver {
            ir::Receiver::Ref => format_ident!("ForwardCall"),
            ir::Receiver::RefMut => format_ident!("ForwardCallMut"),
        };
        let opt_mut = match receiver {
            ir::Receiver::Ref => None,
            ir::Receiver::RefMut => Some(quote! { mut }),
        };
        quote_spanned!(span =>
            type #output_ident = #output_ty;

            #[inline]
            fn #ident( #receiver #(, #inputs_sig )* ) -> Self::#output_ident {
                <&#opt_mut Self as ::ink_lang::#forward_trait>::#forward_ident(self)
                    .#ident( #( #inputs_params ),* )
                    .fire()
                    .expect(#error_str)
            }
        )
    }

    /// Builds up the `ink_env::call::utils::ArgumentList` type structure for the given types.
    pub(super) fn generate_arg_list<'b, Args>(args: Args) -> TokenStream2
    where
        Args: IntoIterator<Item = &'b syn::Type>,
        <Args as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        args.into_iter().fold(
            quote! { ::ink_env::call::utils::EmptyArgumentList },
            |rest, arg| quote! {
                ::ink_env::call::utils::ArgumentList<::ink_env::call::utils::Argument<#arg>, #rest>
            }
        )
    }

    /// Generates the code to allow cross-chain contract calls for trait constructors.
    fn generate_trait_impl_block_constructor(
        &self,
        constructor: ir::InkTraitConstructor,
        selector: ir::Selector,
    ) -> TokenStream2 {
        let span = constructor.span();
        let attrs = constructor.attrs();
        let ident = constructor.ident();
        let output_ident = format_ident!("{}Out", ident.to_string().to_camel_case());
        let selector_bytes = selector.as_bytes().to_owned();
        let input_bindings = constructor
            .inputs()
            .enumerate()
            .map(|(n, _)| format_ident!("__ink_binding_{}", n))
            .collect::<Vec<_>>();
        let input_types = constructor
            .inputs()
            .map(|pat_type| &*pat_type.ty)
            .collect::<Vec<_>>();
        let arg_list = Self::generate_arg_list(input_types.iter().cloned());
        quote_spanned!(span =>
            #[allow(clippy::type_complexity)]
            type #output_ident = ::ink_env::call::CreateBuilder<
                Environment,
                ::ink_env::call::utils::Unset<Hash>,
                ::ink_env::call::utils::Unset<u64>,
                ::ink_env::call::utils::Unset<Balance>,
                ::ink_env::call::utils::Set<::ink_env::call::ExecutionInput<#arg_list>>,
                ::ink_env::call::utils::Unset<::ink_env::call::state::Salt>,
                Self,
            >;

            #( #attrs )*
            #[inline]
            fn #ident(
                #( #input_bindings : #input_types ),*
            ) -> Self::#output_ident {
                ::ink_env::call::build_create::<Environment, Salt, Self>()
                    .exec_input(
                        ::ink_env::call::ExecutionInput::new(
                            ::ink_env::call::Selector::new([ #( #selector_bytes ),* ])
                        )
                        #(
                            .push_arg(#input_bindings)
                        )*
                    )
            }
        )
    }

    /// Returns the identifier of the concrete implementer for the ink! trait.
    fn concrete_implementer_ident(&self) -> syn::Ident {
        let hash = self.trait_def.verify_hash();
        let ident = self.trait_def.ident();
        format_ident!(
            "__ink_ConcreteImplementer{}_0x{:X}{:X}{:X}{:X}",
            ident,
            hash[0],
            hash[1],
            hash[2],
            hash[3]
        )
    }

    /// Generates the short-hand calling implementations for the ink! trait concretizer.
    pub(super) fn generate_trait_impl_block(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let attrs = self.trait_def.attrs();
        let hash = self.trait_def.verify_hash();
        let trait_ident = self.trait_def.ident();
        let self_ident = self.concrete_implementer_ident();
        let verify_hash_id =
            u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]) as usize;
        let messages = self.trait_def.iter_items().filter_map(|(item, selector)| {
            item.filter_map_message().map(|message| {
                self.generate_trait_impl_block_message(message, selector)
            })
        });
        let constructors = self.trait_def.iter_items().filter_map(|(item, selector)| {
            item.filter_map_constructor().map(|constructor| {
                self.generate_trait_impl_block_constructor(constructor, selector)
            })
        });
        quote_spanned!(span =>
            unsafe impl ::ink_lang::CheckedInkTrait<[(); #verify_hash_id]> for #self_ident {}

            #( #attrs )*
            impl #trait_ident for #self_ident {
                type __ink_Checksum = [(); #verify_hash_id];

                #( #messages )*
                #( #constructors )*
            }
        )
    }
}
