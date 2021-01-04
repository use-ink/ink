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

/// Returns `Ok` if there are no occurrences of identifiers starting with `__ink_`.
///
/// # Errors
///
/// Returns a combined error for every instance of `__ink_` prefixed identifier found.
pub fn ensure_no_ink_identifiers<T>(checked: &T) -> Result<(), syn::Error>
where
    T: VisitBy,
{
    let mut visitor = private::IdentVisitor::default();
    checked.visit_by(&mut visitor);
    visitor.into_result()
}

/// Makes sure to call the correct visitor function on the given visitor.
pub trait VisitBy: private::Sealed {
    fn visit_by(&self, visitor: &mut private::IdentVisitor);
}

mod private {
    use super::VisitBy;
    use proc_macro2::Ident;

    /// Seals the implementation of `VisitBy`.
    pub trait Sealed {}
    impl Sealed for syn::ItemMod {}
    impl Sealed for syn::ItemTrait {}
    impl Sealed for syn::ItemFn {}

    impl VisitBy for syn::ItemMod {
        fn visit_by(&self, visitor: &mut IdentVisitor) {
            syn::visit::visit_item_mod(visitor, self);
        }
    }

    impl VisitBy for syn::ItemTrait {
        fn visit_by(&self, visitor: &mut IdentVisitor) {
            syn::visit::visit_item_trait(visitor, self);
        }
    }

    impl VisitBy for syn::ItemFn {
        fn visit_by(&self, visitor: &mut IdentVisitor) {
            syn::visit::visit_item_fn(visitor, self);
        }
    }

    /// Visitor to ensure that there are no identifiers starting with `__ink_` as prefix.
    ///
    /// # Errors
    ///
    /// If there are identifiers starting with `__ink_` as prefix in the input.
    /// Will yield one combined error for all found encounters.
    #[derive(Default)]
    pub struct IdentVisitor {
        errors: Vec<syn::Error>,
    }

    impl IdentVisitor {
        /// Converts the visitor into the errors it found if any.
        ///
        /// Returns `Ok` if it found no errors during visitation.
        pub fn into_result(self) -> Result<(), syn::Error> {
            match self.errors.split_first() {
                None => Ok(()),
                Some((first, rest)) => {
                    let mut combined = first.clone();
                    for error in rest {
                        combined.combine(error.clone());
                    }
                    Err(combined)
                }
            }
        }
    }

    impl<'ast> syn::visit::Visit<'ast> for IdentVisitor {
        fn visit_ident(&mut self, ident: &'ast Ident) {
            if ident.to_string().starts_with("__ink_") {
                self.errors.push(format_err!(
                    ident,
                    "encountered invalid identifier starting with __ink_",
                ))
            }
        }
    }
}
