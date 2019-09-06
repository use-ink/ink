use crate::hir2::data::{
    Contract,
    FnArg,
    Function,
    FunctionKind,
    IdentType,
    Item,
    ItemEvent,
    ItemImpl,
    ItemMeta,
    ItemStorage,
    MetaSimple,
    Signature,
};
use core::convert::TryFrom;
use either::Either;
use itertools::Itertools as _;
use proc_macro2::Ident;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned as _,
    Result,
    Token,
};

impl TryFrom<syn::ItemMod> for Contract {
    type Error = syn::Error;

    fn try_from(item_mod: syn::ItemMod) -> Result<Self> {
        if item_mod.vis != syn::Visibility::Inherited {
            bail!(
                item_mod.vis,
                "contract module must have no visibility modifier",
            )
        }
        let items = match item_mod.content {
            None => {
                bail!(
                    item_mod,
                    "contract module must be inline, e.g. `mod m {{ ... }}`",
                )
            }
            Some((_brace, items)) => items,
        };
        let items = items
            .into_iter()
            .map(Item::try_from)
            .collect::<Result<Vec<_>>>()?;
        let (storage, events, functions) = split_items(items)?;
        let meta_items = item_mod
            .attrs
            .iter()
            .cloned()
            .filter_map(|attr| ItemMeta::try_from(attr).ok())
            .collect::<Vec<_>>();
        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            meta_items,
            attrs: item_mod.attrs,
            storage,
            events,
            functions,
        })
    }
}

impl TryFrom<syn::Attribute> for ItemMeta {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        if !attr.path.is_ident("ink") {
            bail!(attr, "encountered non-ink! meta attribute")
        }
        syn::parse2::<ItemMeta>(attr.tokens)
    }
}

impl Parse for ItemMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let ident = content.parse::<Ident>()?;
        if content.is_empty() {
            return Ok(ItemMeta::Simple(MetaSimple { paren_token, ident }))
        }
        bail_span!(
            paren_token.span,
            "invalid ink! attribute in the given context"
        )
    }
}

impl TryFrom<syn::ItemStruct> for ItemStorage {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if let Some(invalid_meta) = item_struct
            .attrs
            .iter()
            .filter_map(|attr| ItemMeta::try_from(attr.clone()).ok())
            .find(|ink_meta| !ink_meta.is_simple("storage"))
        {
            bail_span!(
                invalid_meta.span(),
                "invalid ink! attribute found for `storage` struct",
            )
        }
        if item_struct.vis != syn::Visibility::Inherited {
            bail!(
                item_struct.vis,
                "visibility modifiers are not allowed for `storage` structs",
            )
        }
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => named_fields,
            syn::Fields::Unnamed(unnamed_fields) => {
                bail!(unnamed_fields, "`storage` structs must have named fields",)
            }
            syn::Fields::Unit => {
                bail!(item_struct, "unit `storage` structs are not allowed",)
            }
        };
        Ok(ItemStorage {
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            attrs: item_struct.attrs,
            fields,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ItemEvent {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if let Some(invalid_meta) = item_struct
            .attrs
            .iter()
            .filter_map(|attr| ItemMeta::try_from(attr.clone()).ok())
            .find(|ink_meta| !ink_meta.is_simple("event"))
        {
            bail_span!(
                invalid_meta.span(),
                "invalid ink! attribute found for `event` struct",
            )
        }
        if item_struct.vis != syn::Visibility::Inherited {
            bail!(
                item_struct,
                "visibility modifiers are not allowed for `event` structs",
            )
        }
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => named_fields,
            syn::Fields::Unnamed(unnamed_fields) => {
                bail!(unnamed_fields, "`event` structs must have named fields",)
            }
            syn::Fields::Unit => {
                bail!(item_struct, "unit `event` structs are not allowed",)
            }
        };
        Ok(ItemEvent {
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            attrs: item_struct.attrs,
            fields,
        })
    }
}

impl TryFrom<syn::ItemImpl> for ItemImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self> {
        if item_impl.defaultness.is_some() {
            bail!(item_impl.defaultness, "`default` not supported in ink!",)
        }
        if item_impl.unsafety.is_some() {
            bail!(
                item_impl.unsafety,
                "`unsafe` implementation blocks are not supported in ink!",
            )
        }
        if !(item_impl.generics.params.is_empty()
            && item_impl.generics.where_clause.is_none())
        {
            bail!(
                item_impl.generics,
                "generic implementation blocks are not supported in ink!",
            )
        }
        if item_impl.trait_.is_some() {
            bail!(item_impl, "trait implementations are not supported in ink!",)
        }
        let type_path = match &*item_impl.self_ty {
            syn::Type::Path(type_path) => type_path,
            _ => {
                bail!(
                    item_impl.self_ty,
                    "encountered invalid ink! implementer type ascription",
                )
            }
        };
        if let Some(qself) = &type_path.qself {
            let span = qself
                .lt_token
                .span()
                .join(qself.gt_token.span())
                .expect("all spans are in the same file; qed");
            bail_span!(
                span,
                "implementation blocks for self qualified paths are not supported in ink!",
            )
        };
        let ident: Ident = match type_path.path.get_ident() {
            Some(ident) => ident.clone(),
            None => {
                bail!(
                    type_path.path,
                    "encountered invalid ink! implementer type path",
                )
            }
        };
        for impl_item in item_impl.items.iter() {
            match impl_item {
                syn::ImplItem::Method(_) => (),
                unsupported_item => {
                    bail!(
                        unsupported_item,
                        "only methods are supported inside impl blocks in ink!",
                    )
                }
            }
        }
        let functions = item_impl
            .items
            .into_iter()
            .filter_map(|impl_item| {
                if let syn::ImplItem::Method(method) = impl_item {
                    return Some(method)
                }
                None
            })
            .map(Function::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            attrs: item_impl.attrs,
            impl_token: item_impl.impl_token,
            self_ty: ident.clone(),
            brace_token: item_impl.brace_token,
            functions,
        })
    }
}

impl TryFrom<syn::ImplItemMethod> for Function {
    type Error = syn::Error;

    fn try_from(method: syn::ImplItemMethod) -> Result<Self> {
        // Partitions ink! attributes into simple and non-simple.
        //
        // Simple attributes are e.g. `#[ink(event)]` that have only
        // one simple identifier in their `(` and `)` body.
        let (simple, non_simple): (Vec<_>, Vec<_>) = method
            .attrs
            .iter()
            .cloned()
            .filter_map(|attr| ItemMeta::try_from(attr).ok())
            .partition_map(|attr| {
                match attr {
                    ItemMeta::Simple(simple) => Either::Left(simple),
                    non_simple => Either::Right(non_simple),
                }
            });
        // Errors if unsupported or unknown non-simple ink! attributes
        // were found for ink! functions.
        if non_simple.len() != 0 {
            return Err(non_simple
                .into_iter()
                .map(|non_simple| {
                    format_err_span!(
                        non_simple.span(),
                        "encountered unsupported ink! attribute for function",
                    )
                })
                .fold1(|mut err1, err2| {
                    err1.combine(err2);
                    err1
                })
                .expect("this must be some since we got at least one error; qed"))
        }
        let mut kind = FunctionKind::Method;
        // Checks for ink! attributes concerning ink! functions.
        //
        // Bails out into error upon unknown or unsupported found ink! attributes.
        // Also errors if some ink! attributes conflict, e.g. if there is a
        // `#[ink(constructor)]` and `#[ink(message)]` attribute or if there is
        // the same attribute multiple times.
        if let Some(err) = simple
            .iter()
            .map(|attr| {
                let new_kind = match attr.ident.to_string().as_str() {
                    "constructor" => Ok(FunctionKind::Constructor),
                    "message" => Ok(FunctionKind::Message),
                    unknown => Err(format_err!(unknown, "unknown ink! attribute found",)),
                }?;
                if kind == FunctionKind::Method {
                    kind = new_kind;
                    Ok(())
                } else {
                    Err(format_err_span!(
                        attr.span(),
                        "conflicting ink! attribute found",
                    ))
                }
            })
            .filter_map(Result::err)
            .fold1(|mut err1, err2| {
                err1.combine(err2);
                err1
            })
        {
            return Err(err)
        }
        // Visibility modifiers are currently not supported for ink! functions.
        if method.vis != syn::Visibility::Inherited {
            bail!(
                method.vis,
                "encountered invalid visibility modifier for ink! function",
            )
        }
        // Functions in ink! must not be `default` since that is unsupported.
        if let Some(defaultness) = method.defaultness {
            bail!(
                defaultness,
                "encountered invalid `default` modifier for ink! function",
            )
        }
        // Check and convert method signature into ink! function signature.
        let sig = Signature::try_from(method.sig)?;
        // Followed by some checks that are depending on the given function kind:
        match kind {
            FunctionKind::Constructor => {
                if !sig.is_mut() {
                    bail_span!(
                        sig.span(),
                        "constructors in ink! must always be `&mut self`",
                    )
                }
                if sig.output != syn::ReturnType::Default {
                    bail!(
                        sig.output,
                        "constructors in ink! must have no specified return type",
                    )
                }
            }
            FunctionKind::Message | FunctionKind::Method => {
                if sig.self_arg().reference.is_none() {
                    bail_span!(
                        sig.span(),
                        "ink! messages and methods must be either `&self` or `&mut self`",
                    )
                }
            }
        }
        // Retain non-ink! attributes only.
        let non_ink_attrs = method
            .attrs
            .into_iter()
            .filter(|attr| ItemMeta::try_from(attr.clone()).is_ok())
            .collect::<Vec<_>>();
        // Finally return the checked ink! function.
        Ok(Self {
            attrs: non_ink_attrs,
            kind,
            sig,
            block: method.block,
        })
    }
}

impl TryFrom<syn::Signature> for Signature {
    type Error = syn::Error;

    fn try_from(sig: syn::Signature) -> Result<Self> {
        if let Some(constness) = sig.constness {
            bail!(constness, "`const fn` is not supported for ink! functions",)
        }
        if let Some(asyncness) = sig.asyncness {
            bail!(asyncness, "`async fn` is not supported for ink! functions",)
        }
        if let Some(unsafety) = sig.unsafety {
            bail!(unsafety, "`unsafe fn` is not supported for ink! functions",)
        }
        if let Some(abi) = sig.abi {
            bail!(abi, "specifying ABI is not allowed in ink! functions",)
        }
        if let Some(variadic) = sig.variadic {
            bail!(
                variadic,
                "variadic functions are not allowed as ink! functions",
            )
        }
        if sig.inputs.len() == 0 {
            bail!(
                sig.inputs,
                "`&self` or `&mut self` is mandatory for ink! functions",
            )
        }
        let inputs = sig
            .inputs
            .iter()
            .cloned()
            .map(FnArg::try_from)
            .collect::<Result<Punctuated<FnArg, Token![,]>>>()?;
        if let FnArg::Typed(ident_type) = &inputs[0] {
            bail_span!(
                ident_type.span(),
                "first argument of ink! functions must be `&self` or `&mut self`",
            )
        }
        for input in inputs.iter().skip(1) {
            match input {
                FnArg::Receiver(receiver) => {
                    bail!(
                        receiver,
                        "unexpected `self` argument found for ink! function",
                    )
                }
                _ => (),
            }
        }
        Ok(Signature {
            fn_token: sig.fn_token,
            ident: sig.ident,
            generics: sig.generics,
            paren_token: sig.paren_token,
            inputs,
            output: sig.output,
        })
    }
}

impl TryFrom<syn::FnArg> for FnArg {
    type Error = syn::Error;

    fn try_from(fn_arg: syn::FnArg) -> Result<Self> {
        match fn_arg {
            syn::FnArg::Receiver(receiver) => Ok(FnArg::Receiver(receiver)),
            syn::FnArg::Typed(pat_type) => {
                match *pat_type.pat {
                    syn::Pat::Ident(pat_ident) => {
                        if let Some(by_ref) = pat_ident.by_ref {
                            bail!(
                            by_ref,
                            "`ref` modifier is unsupported for ink! function arguments",
                        )
                        }
                        Ok(FnArg::Typed(IdentType {
                            attrs: pat_ident.attrs,
                            ident: pat_ident.ident,
                            colon_token: pat_type.colon_token,
                            ty: *pat_type.ty,
                        }))
                    }
                    unsupported => bail!(
                    unsupported,
                    "encountered unsupported function argument syntax for ink! function",
                ),
                }
            }
        }
    }
}

impl TryFrom<syn::Item> for Item {
    type Error = syn::Error;

    fn try_from(item: syn::Item) -> Result<Self> {
        match item {
            syn::Item::Impl(item_impl) => ItemImpl::try_from(item_impl).map(Into::into),
            syn::Item::Struct(item_struct) => {
                // Can either be a storage or event struct.
                let ink_attrs = item_struct
                    .attrs
                    .iter()
                    .cloned()
                    .filter_map(|attr| ItemMeta::try_from(attr).ok())
                    .collect::<Vec<_>>();
                if ink_attrs.iter().any(|meta| meta.is_simple("event")) {
                    return ItemEvent::try_from(item_struct).map(Into::into)
                }
                // Users can leave away `#[ink(storage)]` so this can be the fallback.
                return ItemStorage::try_from(item_struct).map(Into::into)
            }
            unsupported => {
                bail!(unsupported, "encountered unsupported contract module item",)
            }
        }
    }
}

/// Split the storage, the events and functions out of the general contract items vector.
///
/// # Erros
///
/// - When there is no storage struct.
/// - When a contract item is invalid.
fn split_items(items: Vec<Item>) -> Result<(ItemStorage, Vec<ItemEvent>, Vec<Function>)> {
    let (mut storages, non_storage_items): (Vec<ItemStorage>, Vec<Item>) =
        items.into_iter().partition_map(|item| {
            match item {
                Item::Storage(item_storage) => Either::Left(item_storage),
                other => Either::Right(other),
            }
        });
    let storage = if storages.len() != 1 {
        Err(storages
            .iter()
            .map(|storage| format_err!(storage.ident, "conflicting storage struct found"))
            .fold1(|mut err1, err2| {
                err1.combine(err2);
                err1
            })
            .expect("there must be at least 2 conflicting storages; qed"))
    } else {
        Ok(storages
            .pop()
            .expect("there must be exactly one storage in `storages`"))
    }?;
    let (events, impl_blocks): (Vec<ItemEvent>, Vec<ItemImpl>) =
        non_storage_items.into_iter().partition_map(|item| {
            match item {
                Item::Event(item_event) => Either::Left(item_event),
                Item::Impl(item_impl) => Either::Right(item_impl),
                Item::Storage(_) => {
                    unreachable!(
                        "we should not have any storages left at this point; qed"
                    )
                }
                _ => unreachable!("we do not expect any other kinds of items"),
            }
        });
    let functions = impl_blocks
        .into_iter()
        .map(|impl_block| impl_block.functions)
        .flatten()
        .collect::<Vec<_>>();
    Ok((storage, events, functions))
}
