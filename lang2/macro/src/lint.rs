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

use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
    TokenTree as TokenTree2,
};

/// Returns `Ok` if all identifiers within the `TokenStream` match `pred`
/// and otherwise return an error pointing to the faulty `Ident`.
pub fn idents_respect_pred<P, E>(
    input: TokenStream2,
    pred: P,
    or_err: E,
) -> Result<(), syn::Error>
where
    P: Copy + Fn(&Ident) -> bool,
    E: Copy + Fn(&Ident) -> syn::Error,
{
    for tt in input.into_iter() {
        match tt {
            TokenTree2::Ident(ident) => {
                if !pred(&ident) {
                    return Err(or_err(&ident))
                }
            }
            TokenTree2::Group(group) => {
                // We ignore upon success and return back upon error.
                idents_respect_pred(group.stream(), pred, or_err)?;
            }
            TokenTree2::Punct(_) | TokenTree2::Literal(_) => (),
        }
    }
    Ok(())
}
