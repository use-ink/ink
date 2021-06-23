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

use crate::format_err;
use proc_macro2::Span;
use syn::spanned::Spanned as _;

/// Ensures that the given visibility is `pub` and otherwise returns an appropriate error.
///
/// # Note
///
/// The `name` parameter is given to improve the resulting error message. It denotes the
/// entity which cannot have non-public visibility.
pub fn ensure_pub_visibility(
    name: &str,
    parent_span: Span,
    vis: &syn::Visibility,
) -> Result<(), syn::Error> {
    let bad_visibility = match vis {
        syn::Visibility::Inherited => Some(parent_span),
        syn::Visibility::Restricted(vis_restricted) => Some(vis_restricted.span()),
        syn::Visibility::Crate(vis_crate) => Some(vis_crate.span()),
        syn::Visibility::Public(_) => None,
    };
    if let Some(bad_visibility) = bad_visibility {
        return Err(format_err!(
            bad_visibility,
            "non `pub` ink! {} are not supported",
            name
        ))
    }
    Ok(())
}
