use crate::ir;
use core::convert::TryFrom;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    spanned::Spanned as _,
    Result,
};

#[derive(Debug, PartialEq, Eq)]
pub struct InkTrait {
    item: syn::ItemTrait,
}

impl TryFrom<syn::ItemTrait> for InkTrait {
    type Error = syn::Error;

    fn try_from(item_trait: syn::ItemTrait) -> core::result::Result<Self, Self::Error> {
        Self::analyse_properties(&item_trait)?;
        Self::analyse_items(&item_trait)?;
        Ok(Self { item: item_trait })
    }
}

impl InkTrait {
    /// Returns `Ok` if the trait matches all requirements for an ink! trait definition.
    pub fn new(attr: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
        if !attr.is_empty() {
            return Err(format_err_spanned!(
                attr,
                "unexpected attribute input for ink! trait definition"
            ))
        }
        let item_trait = syn::parse2::<syn::ItemTrait>(input.clone())?;
        let _ink_trait = InkTrait::try_from(item_trait)?;
        Ok(input)
    }

    /// Analyses the properties of the ink! trait definition.
    ///
    /// # Errors
    ///
    /// - If the trait has been defined as `unsafe`.
    /// - If the trait is an automatically implemented trait (`auto trait`).
    /// - If the trait is generic over some set of types.
    /// - If the trait's visibility is not public (`pub`).
    fn analyse_properties(item_trait: &syn::ItemTrait) -> Result<()> {
        if let Some(unsafety) = &item_trait.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "ink! trait definitions cannot be unsafe"
            ))
        }
        if let Some(auto) = &item_trait.auto_token {
            return Err(format_err_spanned!(
                auto,
                "ink! trait definitions cannot be automatically implemented traits"
            ))
        }
        if !item_trait.generics.params.is_empty() {
            return Err(format_err_spanned!(
                item_trait.generics.params,
                "ink! trait definitions must not be generic"
            ))
        }
        if !matches!(item_trait.vis, syn::Visibility::Public(_)) {
            return Err(format_err_spanned!(
                item_trait.vis,
                "ink! trait definitions must have public visibility"
            ))
        }
        Ok(())
    }

    /// Returns `Ok` if all trait items respects the requirements for an ink! trait definition.
    ///
    /// # Errors
    ///
    /// - If the trait contains an unsupported trait item such as
    ///     - associated constants (`const`)
    ///     - associated types (`type`)
    ///     - macros definitions or usages
    ///     - unknown token sequences (verbatims)
    ///     - methods with default implementations
    /// - If the trait contains methods which do not respect the ink! trait definition requirements:
    ///     - All trait methods need to be declared as either `#[ink(message)]` or `#[ink(constructor)]`
    ///       and need to respect their respective rules.
    ///
    /// # Note
    ///
    /// Associated types and constants might be allowed in the future.
    fn analyse_items(item_trait: &syn::ItemTrait) -> Result<()> {
        for trait_item in &item_trait.items {
            match trait_item {
                syn::TraitItem::Const(const_trait_item) => {
                    return Err(format_err_spanned!(
                        const_trait_item,
                        "associated constants in ink! trait definitions are not supported, yet"
                    ))
                }
                syn::TraitItem::Macro(macro_trait_item) => {
                    return Err(format_err_spanned!(
                        macro_trait_item,
                        "macros in ink! trait definitions are not supported"
                    ))
                }
                syn::TraitItem::Type(type_trait_item) => {
                    return Err(format_err_spanned!(
                    type_trait_item,
                    "associated types in ink! trait definitions are not supported, yet"
                ))
                }
                syn::TraitItem::Verbatim(verbatim) => {
                    return Err(format_err_spanned!(
                        verbatim,
                        "encountered unsupported item in ink! trait definition"
                    ))
                }
                syn::TraitItem::Method(method_trait_item) => {
                    Self::analyse_methods(method_trait_item)?;
                }
                unknown => {
                    return Err(format_err_spanned!(
                        unknown,
                        "encountered unknown or unsupported item in ink! trait definition"
                    ))
                }
            }
        }
        Ok(())
    }

    /// Analyses an ink! method that can be either an ink! message or constructor.
    ///
    /// # Errors
    ///
    /// - If the method declared as `unsafe`, `const` or `async`.
    /// - If the method has some explicit API.
    /// - If the method is variadic or has generic parameters.
    /// - If the method does not respect the properties of either an
    ///   ink! message or ink! constructor.
    fn analyse_methods(method: &syn::TraitItemMethod) -> Result<()> {
        if let Some(default_impl) = &method.default {
            return Err(format_err_spanned!(
                default_impl,
                "ink! trait methods with default implementations are not supported"
            ))
        }
        if let Some(constness) = &method.sig.constness {
            return Err(format_err_spanned!(
                constness,
                "const ink! trait methods are not supported"
            ))
        }
        if let Some(asyncness) = &method.sig.asyncness {
            return Err(format_err_spanned!(
                asyncness,
                "async ink! trait methods are not supported"
            ))
        }
        if let Some(unsafety) = &method.sig.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "unsafe ink! trait methods are not supported"
            ))
        }
        if let Some(abi) = &method.sig.abi {
            return Err(format_err_spanned!(
                abi,
                "ink! trait methods with non default ABI are not supported"
            ))
        }
        if let Some(variadic) = &method.sig.variadic {
            return Err(format_err_spanned!(
                variadic,
                "variadic ink! trait methods are not supported"
            ))
        }
        if !method.sig.generics.params.is_empty() {
            return Err(format_err_spanned!(
                method.sig.generics.params,
                "generic ink! trait methods are not supported"
            ))
        }
        match ir::first_ink_attribute(&method.attrs) {
            Ok(Some(ink_attr)) => {
                match ink_attr.first().kind() {
                    ir::AttributeArgKind::Message => {
                        Self::analyse_message(method)?;
                    }
                    ir::AttributeArgKind::Constructor => {
                        Self::analyse_constructor(method)?;
                    }
                    _unsupported => {
                        return Err(format_err_spanned!(
                            method,
                            "encountered unsupported ink! attribute for ink! trait method",
                        ))
                    }
                }
            }
            Ok(None) => {
                return Err(format_err_spanned!(
                    method,
                    "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method"
                ))
            }
            Err(err) => return Err(err),
        }
        Ok(())
    }

    /// Analyses the properties of an ink! constructor.
    ///
    /// # Errors
    ///
    /// - If the constructor has a `self` receiver as first argument.
    /// - If the constructor has no `Self` return type.
    fn analyse_constructor(constructor: &syn::TraitItemMethod) -> Result<()> {
        ir::sanitize_attributes(
            constructor.span(),
            constructor.attrs.clone(),
            &ir::AttributeArgKind::Constructor,
            |c| {
                match c {
                    ir::AttributeArgKind::Constructor => false,
                    _ => true,
                }
            },
        )?;
        match &constructor.sig.inputs.first() {
            None => (),
            Some(syn::FnArg::Typed(pat_type)) => {
                match &*pat_type.ty {
                    syn::Type::Path(type_path) => {
                        if type_path.path.is_ident("Self") {
                            return Err(format_err_spanned!(
                                type_path.path,
                                "encountered invalid `Self` receiver for ink! constructor"
                            ))
                        }
                    }
                    syn::Type::Reference(type_reference) => {
                        if let syn::Type::Path(type_path) = &*type_reference.elem {
                            if type_path.path.is_ident("Self") {
                                return Err(format_err_spanned!(
                                    type_path.path,
                                    "encountered invalid `Self` receiver for ink! constructor"
                                ))
                            }
                        }
                    }
                    _ => (),
                }
            }
            Some(syn::FnArg::Receiver(receiver)) => {
                return Err(format_err_spanned!(
                    receiver,
                    "ink! constructors must not have a `self` receiver",
                ))
            }
        }
        match &constructor.sig.output {
            syn::ReturnType::Default => {
                return Err(format_err_spanned!(
                    constructor.sig,
                    "ink! constructors must return Self"
                ))
            }
            syn::ReturnType::Type(_, ty) => {
                match &**ty {
                    syn::Type::Path(type_path) => {
                        if !type_path.path.is_ident("Self") {
                            return Err(format_err_spanned!(
                                type_path.path,
                                "ink! constructors must return Self"
                            ))
                        }
                    }
                    unknown => {
                        return Err(format_err_spanned!(
                            unknown,
                            "ink! constructors must return Self"
                        ))
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyses the properties of an ink! message.
    ///
    /// # Errors
    ///
    /// - If the message has no `&self` or `&mut self` receiver.
    fn analyse_message(message: &syn::TraitItemMethod) -> Result<()> {
        ir::sanitize_attributes(
            message.span(),
            message.attrs.clone(),
            &ir::AttributeArgKind::Message,
            |c| {
                match c {
                    ir::AttributeArgKind::Message => false,
                    _ => true,
                }
            },
        )?;
        match message.sig.inputs.first() {
            None | Some(syn::FnArg::Typed(_)) => {
                return Err(format_err_spanned!(
                    message.sig,
                    "missing `&self` or `&mut self` receiver for ink! message",
                ))
            }
            Some(syn::FnArg::Receiver(receiver)) => {
                if receiver.reference.is_none() {
                    return Err(format_err_spanned!(
                        receiver,
                        "self receiver of ink! message must be `&self` or `&mut self`"
                    ))
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks if the token stream in `$trait_def` results in the expected error message.
    macro_rules! assert_ink_trait_eq_err {
        ( error: $err_str:literal, $($trait_def:tt)* ) => {
            assert_eq!(
                <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                    $( $trait_def )*
                })
                .map_err(|err| err.to_string()),
                Err(
                    $err_str.to_string()
                )
            )
        };
    }

    #[test]
    fn unsafe_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions cannot be unsafe",
            pub unsafe trait MyTrait {}
        );
    }

    #[test]
    fn auto_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions cannot be automatically implemented traits",
            pub auto trait MyTrait {}
        );
    }

    #[test]
    fn non_pub_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions must have public visibility",
            trait MyTrait {}
        );
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions must have public visibility",
            pub(crate) trait MyTrait {}
        );
    }

    #[test]
    fn generic_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions must not be generic",
            trait MyTrait<T> {}
        );
    }

    #[test]
    fn trait_def_containing_const_item_is_denied() {
        assert_ink_trait_eq_err!(
            error: "associated constants in ink! trait definitions are not supported, yet",
            pub trait MyTrait {
                const T: i32;
            }
        );
    }

    #[test]
    fn trait_def_containing_associated_type_is_denied() {
        assert_ink_trait_eq_err!(
            error: "associated types in ink! trait definitions are not supported, yet",
            pub trait MyTrait {
                type Type;
            }
        );
    }

    #[test]
    fn trait_def_containing_macro_is_denied() {
        assert_ink_trait_eq_err!(
            error: "macros in ink! trait definitions are not supported",
            pub trait MyTrait {
                my_macro_call!();
            }
        );
    }

    #[test]
    fn trait_def_containing_non_flagged_method_is_denied() {
        assert_ink_trait_eq_err!(
            error: "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method",
            pub trait MyTrait {
                fn non_flagged_1(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method",
            pub trait MyTrait {
                fn non_flagged_2(&mut self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method",
            pub trait MyTrait {
                fn non_flagged_3() -> Self;
            }
        );
    }

    #[test]
    fn trait_def_containing_default_implemented_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with default implementations are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                fn default_implemented() -> Self {}
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with default implementations are not supported",
            pub trait MyTrait {
                #[ink(message)]
                fn default_implemented(&self) {}
            }
        );
    }

    #[test]
    fn trait_def_containing_const_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "const ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                const fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "const ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                const fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_async_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "async ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                async fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "async ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                async fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_unsafe_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "unsafe ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                unsafe fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "unsafe ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                unsafe fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_methods_using_explicit_abi_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with non default ABI are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                extern fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with non default ABI are not supported",
            pub trait MyTrait {
                #[ink(message)]
                extern fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_variadic_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "variadic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                fn const_constructor(...) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "variadic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                fn const_message(&self, ...);
            }
        );
    }

    #[test]
    fn trait_def_containing_generic_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "generic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                fn const_constructor<T>() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "generic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                fn const_message<T>(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_method_with_unsupported_ink_attribute_is_denied() {
        assert_ink_trait_eq_err!(
            error: "encountered unsupported ink! attribute for ink! trait method",
            pub trait MyTrait {
                #[ink(payable)]
                fn unsupported_ink_attribute(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "unknown ink! attribute (path)",
            pub trait MyTrait {
                #[ink(unknown)]
                fn unknown_ink_attribute(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_invalid_constructor_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(&self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(&mut self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered invalid `Self` receiver for ink! constructor",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(this: &Self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered invalid `Self` receiver for ink! constructor",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(this: Self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must return Self",
            pub trait MyTrait {
                #[ink(constructor)]
                fn does_not_return_self();
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must return Self",
            pub trait MyTrait {
                #[ink(constructor)]
                fn does_not_return_self() -> i32;
            }
        );
    }

    #[test]
    fn trait_def_containing_invalid_message_is_denied() {
        assert_ink_trait_eq_err!(
            error: "missing `&self` or `&mut self` receiver for ink! message",
            pub trait MyTrait {
                #[ink(message)]
                fn does_not_return_self();
            }
        );
        assert_ink_trait_eq_err!(
            error: "self receiver of ink! message must be `&self` or `&mut self`",
            pub trait MyTrait {
                #[ink(message)]
                fn does_not_return_self(self);
            }
        );
    }

    #[test]
    fn trait_def_containing_constructor_with_invalid_ink_attributes_is_denied() {
        assert_ink_trait_eq_err!(
            error: "encountered duplicate ink! attribute",
            pub trait MyTrait {
                #[ink(constructor)]
                #[ink(constructor)]
                fn does_not_return_self() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(constructor)]
                #[ink(message)]
                fn does_not_return_self() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(constructor)]
                #[ink(payable)]
                fn does_not_return_self() -> Self;
            }
        );
    }

    #[test]
    fn trait_def_containing_message_with_invalid_ink_attributes_is_denied() {
        assert_ink_trait_eq_err!(
            error: "encountered duplicate ink! attribute",
            pub trait MyTrait {
                #[ink(message)]
                #[ink(message)]
                fn does_not_return_self(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(message)]
                #[ink(constructor)]
                fn does_not_return_self(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(message)]
                #[ink(payable)]
                fn does_not_return_self(&self);
            }
        );
    }

    #[test]
    fn trait_def_is_ok() {
        assert!(
            <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                pub trait MyTrait {
                    #[ink(constructor)]
                    fn my_constructor() -> Self;
                    #[ink(message)]
                    fn my_message(&self);
                    #[ink(message)]
                    fn my_message_mut(&mut self);
                }
            })
            .is_ok()
        )
    }
}
