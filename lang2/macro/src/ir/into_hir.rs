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

//! Contains all converion routines from Rust AST to ink! IR.

use crate::ir::{
    Contract,
    FnArg,
    Function,
    FunctionKind,
    FunctionSelector,
    IdentType,
    Item,
    ItemEvent,
    ItemImpl,
    ItemStorage,
    KindConstructor,
    KindMessage,
    Marker,
    MetaInfo,
    MetaParam,
    MetaTypes,
    ParamTypes,
    Params,
    Signature,
    SimpleMarker,
};
use core::convert::TryFrom;
use either::Either;
use itertools::Itertools as _;
use proc_macro2::Ident;
use std::collections::HashSet;
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

impl Parse for Marker {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let ident = content.parse::<Ident>()?;
        if content.is_empty() {
            return Ok(Marker::Simple(SimpleMarker { paren_token, ident }))
        }
        bail_span!(
            paren_token.span,
            "invalid ink! attribute in the given context",
        )
    }
}

impl TryFrom<(Params, syn::ItemMod)> for Contract {
    type Error = syn::Error;

    fn try_from((params, item_mod): (Params, syn::ItemMod)) -> Result<Self> {
        if item_mod.vis != syn::Visibility::Inherited {
            bail!(
                item_mod.vis,
                "contract module must have no visibility modifier",
            )
        }
        let items = match &item_mod.content {
            None => {
                bail!(
                    item_mod,
                    "contract module must be inline, e.g. `mod m {{ ... }}`",
                )
            }
            Some((_brace, items)) => items.clone(),
        };
        let items = items
            .into_iter()
            .map(Item::try_from)
            .collect::<Result<Vec<_>>>()?;
        let (storage, events, functions) = split_items(items)?;
        if functions.iter().filter(|f| f.is_constructor()).count() == 0 {
            bail!(
                &item_mod,
                "ink! contracts require at least one constructor function declared with `#[ink(constructor)]`",
            )
        }
        let meta_info = MetaInfo::try_from(params)?;
        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            attrs: item_mod.attrs,
            meta_info,
            storage,
            events,
            functions,
        })
    }
}

impl TryFrom<Params> for MetaInfo {
    type Error = syn::Error;

    fn try_from(params: Params) -> Result<Self> {
        let mut unique_params = HashSet::new();
        let mut env_types = None;
        let mut ink_version = None;
        for param in params.params.iter().cloned() {
            let name = param.ident().to_string();
            if !unique_params.insert(name) {
                bail_span!(param.span(), "encountered parameter multiple times",)
            }
            match param {
                MetaParam::Types(param_types) => {
                    env_types = Some(MetaTypes::try_from(param_types)?)
                }
                MetaParam::Version(param_version) => {
                    ink_version = Some(param_version.data)
                }
            }
        }
        match (env_types, ink_version) {
            (None, _) => {
                bail_span!(
                    params.span(),
                    "expected `types` argument at `#[ink::contract(..)]`",
                )
            }
            (_, None) => {
                bail_span!(
                    params.span(),
                    "expected `version` argument at `#[ink::contract(..)]`",
                )
            }
            (Some(env_types), Some(ink_version)) => {
                Ok(Self {
                    env_types,
                    ink_version,
                })
            }
        }
    }
}

impl TryFrom<ParamTypes> for MetaTypes {
    type Error = syn::Error;

    fn try_from(params: ParamTypes) -> Result<Self> {
        Ok(Self { ty: params.ty })
    }
}

impl TryFrom<syn::Attribute> for Marker {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        if !attr.path.is_ident("ink") {
            bail!(attr, "encountered non-ink! attribute")
        }
        syn::parse2::<Self>(attr.tokens)
    }
}

impl TryFrom<syn::ItemStruct> for ItemStorage {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if let Some(invalid_meta) = item_struct
            .attrs
            .iter()
            .filter_map(|attr| Marker::try_from(attr.clone()).ok())
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
        let span = item_struct.span();
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
            span,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ItemEvent {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if let Some(invalid_meta) = item_struct
            .attrs
            .iter()
            .filter_map(|attr| Marker::try_from(attr.clone()).ok())
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
        let span = method.span();
        // Partitions ink! attributes into simple and non-simple.
        //
        // Simple attributes are e.g. `#[ink(event)]` that have only
        // one simple identifier in their `(` and `)` body.
        let simple = method
            .attrs
            .iter()
            .cloned()
            .filter_map(|attr| Marker::try_from(attr).ok())
            .map(|attr| {
                match attr {
                    Marker::Simple(simple) => simple,
                }
            });
        // Checks for ink! attributes concerning ink! functions.
        //
        // Bails out into error upon unknown or unsupported found ink! attributes.
        // Also errors if some ink! attributes conflict, e.g. if there is a
        // `#[ink(constructor)]` and `#[ink(message)]` attribute or if there is
        // the same attribute multiple times.
        let mut kind = FunctionKind::Method;
        if let Some(err) = simple
            .map(|attr| {
                let new_kind = match attr.ident.to_string().as_str() {
                    "constructor" => {
                        Ok(FunctionKind::Constructor(KindConstructor {
                            selector: FunctionSelector::from(&method.sig.ident),
                        }))
                    }
                    "message" => {
                        Ok(FunctionKind::Message(KindMessage {
                            selector: FunctionSelector::from(&method.sig.ident),
                        }))
                    }
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
            FunctionKind::Constructor(_) => {
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
            FunctionKind::Message(_) | FunctionKind::Method => {
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
            .filter(|attr| Marker::try_from(attr.clone()).is_err())
            .collect::<Vec<_>>();
        // Finally return the checked ink! function.
        Ok(Self {
            attrs: non_ink_attrs,
            kind,
            sig,
            block: method.block,
            span,
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
                    unsupported => {
                        bail!(
                    unsupported,
                    "encountered unsupported function argument syntax for ink! function",
                )
                    }
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
                    .filter_map(|attr| Marker::try_from(attr).ok())
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
            }
        });
    let functions = impl_blocks
        .into_iter()
        .map(|impl_block| impl_block.functions)
        .flatten()
        .collect::<Vec<_>>();
    Ok((storage, events, functions))
}
