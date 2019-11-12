// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

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
