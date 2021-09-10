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

use quote::{quote, quote_spanned};
use proc_macro2::TokenStream as TokenStream2;

/// Builds up the `ink_env::call::utils::ArgumentList` type structure for the given types.
pub fn generate_argument_list<'b, Args>(args: Args) -> TokenStream2
where
    Args: IntoIterator<Item = &'b syn::Type>,
    <Args as IntoIterator>::IntoIter: Iterator,
{
    use syn::spanned::Spanned as _;
    args.into_iter().fold(
        quote! { ::ink_env::call::utils::EmptyArgumentList },
        |rest, arg| {
            let span = arg.span();
            quote_spanned!(span=>
                ::ink_env::call::utils::ArgumentList<::ink_env::call::utils::Argument<#arg>, #rest>
            )
        }
    )
}
