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
    Span,
};

/// Utilities for operating on `Ident` instances.
pub trait IdentExt {
    /// Creates a new Ident from the given `str`.
    fn from_str<T: AsRef<str>>(s: T) -> Ident {
        Ident::new(s.as_ref(), Span::call_site())
    }
}

impl IdentExt for Ident {}
