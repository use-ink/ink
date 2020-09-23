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

//! Contains all conversion routines from Rust AST to ink! IR.

use crate::{
    ir,
    ir::utils,
};
use core::convert::TryFrom;
use either::Either;
use ink_lang_ir::{
    format_err,
    format_err_spanned,
};
use itertools::Itertools as _;
use proc_macro2::{
    Ident,
    Span,
};
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

impl Parse for ir::Marker {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let ident = content.parse::<Ident>()?;
        if content.is_empty() {
            return Ok(ir::Marker::Simple(ir::SimpleMarker { paren_token, ident }))
        }
        Err(format_err!(
            paren_token.span,
            "invalid ink! attribute in the given context",
        ))
    }
}

impl TryFrom<(ir::Params, syn::ItemFn)> for ir::InkTest {
    type Error = syn::Error;

    fn try_from((_params, item_fn): (ir::Params, syn::ItemFn)) -> Result<Self> {
        Ok(Self { item_fn })
    }
}

impl TryFrom<(ir::Params, syn::ItemMod)> for ir::Contract {
    type Error = syn::Error;

    fn try_from((params, item_mod): (ir::Params, syn::ItemMod)) -> Result<Self> {
        if item_mod.vis != syn::Visibility::Inherited {
            return Err(format_err_spanned!(
                item_mod.vis,
                "contract module must have no visibility modifier",
            ))
        }
        let items = match &item_mod.content {
            None => {
                return Err(format_err_spanned!(
                    item_mod,
                    "contract module must be inline, e.g. `mod m {{ ... }}`",
                ))
            }
            Some((_brace, items)) => items.clone(),
        };
        use itertools::Itertools as _;
        let (ink_items, rust_items): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(ir::Item::try_from)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .partition_map(|item| {
                match item {
                    ir::Item::Ink(ink_item) => Either::Left(ink_item),
                    ir::Item::Rust(rust_item) => Either::Right(rust_item),
                }
            });
        let (storage, events, functions) = split_items(ink_items)?;
        if functions.iter().filter(|f| f.is_constructor()).count() == 0 {
            return Err(format_err_spanned!(
                &item_mod,
                "ink! contracts require at least one `#[ink(constructor)]`"
            ))
        }
        if functions.iter().filter(|f| f.is_message()).count() == 0 {
            return Err(format_err_spanned!(
                &item_mod,
                "ink! contracts require at least one `#[ink(message)]`"
            ))
        }
        let meta_info = ir::MetaInfo::try_from(params)?;
        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            attrs: item_mod.attrs,
            meta_info,
            storage,
            events,
            functions,
            non_ink_items: rust_items,
        })
    }
}

impl TryFrom<ir::Params> for ir::MetaInfo {
    type Error = syn::Error;

    fn try_from(params: ir::Params) -> Result<Self> {
        let mut unique_params = HashSet::new();
        let mut env_types = None;
        let mut ink_version = None;
        let mut dynamic_allocations: Option<bool> = None;
        let mut compile_as_dependency: Option<bool> = None;
        for param in params.params.iter().cloned() {
            let name = param.ident().to_string();
            if !unique_params.insert(name) {
                return Err(format_err!(
                    param.span(),
                    "encountered parameter multiple times",
                ))
            }
            match param {
                ir::MetaParam::Types(param) => {
                    env_types = Some(ir::MetaTypes::try_from(param)?)
                }
                ir::MetaParam::Version(param) => ink_version = Some(param.data),
                ir::MetaParam::DynamicAllocations(param) => {
                    dynamic_allocations = Some(param.value.value)
                }
                ir::MetaParam::CompileAsDependency(param) => {
                    compile_as_dependency = Some(param.value.value)
                }
            }
        }
        let ink_version = match ink_version {
            None => {
                return Err(format_err!(
                    params.span(),
                    "expected `version` argument at `#[ink::contract(..)]`",
                ))
            }
            Some(ink_version) => ink_version,
        };
        Ok(Self {
            env_types: env_types.unwrap_or_default(),
            ink_version,
            dynamic_allocations_enabled: dynamic_allocations.unwrap_or(false),
            compile_as_dependency: compile_as_dependency.unwrap_or(false),
        })
    }
}

impl TryFrom<ir::ParamTypes> for ir::MetaTypes {
    type Error = syn::Error;

    fn try_from(params: ir::ParamTypes) -> Result<Self> {
        Ok(Self { ty: params.ty })
    }
}

impl TryFrom<syn::Attribute> for ir::Marker {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        if !attr.path.is_ident("ink") {
            return Err(format_err_spanned!(attr, "encountered non-ink! attribute"))
        }
        syn::parse2::<Self>(attr.tokens)
    }
}

impl TryFrom<syn::ItemStruct> for ir::ItemStorage {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if let Some(invalid_meta) = item_struct
            .attrs
            .iter()
            .filter_map(|attr| ir::Marker::try_from(attr.clone()).ok())
            .find(|ink_meta| !ink_meta.is_simple("storage"))
        {
            return Err(format_err!(
                invalid_meta.span(),
                "invalid ink! attribute found for `#[ink(storage)]` struct",
            ))
        }
        if item_struct.vis != syn::Visibility::Inherited {
            return Err(format_err_spanned!(
                item_struct.vis,
                "visibility modifiers are not allowed for `#[ink(storage)]` structs",
            ))
        }
        let span = item_struct.span();
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => named_fields,
            syn::Fields::Unnamed(unnamed_fields) => {
                return Err(format_err_spanned!(
                    unnamed_fields,
                    "`#[ink(storage)]` tuple-structs are forbidden"
                ))
            }
            syn::Fields::Unit => {
                return Err(format_err_spanned!(
                    item_struct,
                    "`#[ink(storage)]` unit-structs are forbidden"
                ))
            }
        };
        Ok(ir::ItemStorage {
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            attrs: item_struct.attrs,
            fields,
            span,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ir::ItemEvent {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if let Some(invalid_meta) = item_struct
            .attrs
            .iter()
            .filter_map(|attr| ir::Marker::try_from(attr.clone()).ok())
            .find(|ink_meta| !ink_meta.is_simple("event"))
        {
            return Err(format_err!(
                invalid_meta.span(),
                "invalid ink! attribute found for `#[ink(event)]` struct",
            ))
        }
        if item_struct.vis != syn::Visibility::Inherited {
            return Err(format_err_spanned!(
                item_struct,
                "visibility modifiers are not allowed for `#[ink(event)]` structs",
            ))
        }
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => named_fields,
            syn::Fields::Unnamed(unnamed_fields) => {
                return Err(format_err_spanned!(
                    unnamed_fields,
                    "`#[ink(event)]` tuple-structs are forbidden",
                ))
            }
            syn::Fields::Unit => {
                return Err(format_err_spanned!(
                    item_struct,
                    "`#[ink(event)]` unit-structs are forbidden",
                ))
            }
        };
        Ok(ir::ItemEvent {
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            attrs: item_struct.attrs,
            fields,
        })
    }
}

impl TryFrom<syn::ItemImpl> for ir::ItemImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self> {
        if item_impl.defaultness.is_some() {
            return Err(format_err_spanned!(
                item_impl.defaultness,
                "`default` not supported in ink!",
            ))
        }
        if item_impl.unsafety.is_some() {
            return Err(format_err_spanned!(
                item_impl.unsafety,
                "`unsafe` implementation blocks are not supported in ink!",
            ))
        }
        if !(item_impl.generics.params.is_empty()
            && item_impl.generics.where_clause.is_none())
        {
            return Err(format_err_spanned!(
                item_impl.generics,
                "generic implementation blocks are not supported in ink!",
            ))
        }
        if item_impl.trait_.is_some() {
            return Err(format_err_spanned!(
                item_impl,
                "trait implementations are not supported in ink!",
            ))
        }
        let type_path = match &*item_impl.self_ty {
            syn::Type::Path(type_path) => type_path,
            _ => {
                return Err(format_err_spanned!(
                    item_impl.self_ty,
                    "encountered invalid ink! implementer type ascription",
                ))
            }
        };
        if let Some(qself) = &type_path.qself {
            let span = qself
                .lt_token
                .span()
                .join(qself.gt_token.span())
                .expect("all spans are in the same file; qed");
            return Err(format_err!(
                span,
                "implementation blocks for self qualified paths are not supported in ink!",
            ))
        };
        let ident: Ident = match type_path.path.get_ident() {
            Some(ident) => ident.clone(),
            None => {
                return Err(format_err_spanned!(
                    type_path.path,
                    "encountered invalid ink! implementer type path",
                ))
            }
        };
        for impl_item in item_impl.items.iter() {
            match impl_item {
                syn::ImplItem::Method(_) => (),
                unsupported_item => {
                    return Err(format_err_spanned!(
                        unsupported_item,
                        "only methods are supported inside impl blocks in ink!",
                    ))
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
            .map(ir::Function::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            attrs: item_impl.attrs,
            impl_token: item_impl.impl_token,
            self_ty: ident,
            brace_token: item_impl.brace_token,
            functions,
        })
    }
}

impl TryFrom<syn::ImplItemMethod> for ir::Function {
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
            .filter_map(|attr| ir::Marker::try_from(attr).ok())
            .map(|attr| {
                match attr {
                    ir::Marker::Simple(simple) => simple,
                }
            });
        // Checks for ink! attributes concerning ink! functions.
        //
        // Bails out into error upon unknown or unsupported found ink! attributes.
        // Also errors if some ink! attributes conflict, e.g. if there is a
        // `#[ink(constructor)]` and `#[ink(message)]` attribute or if there is
        // the same attribute multiple times.
        let mut kind = ir::FunctionKind::Method;
        if let Some(err) = simple
            .map(|attr| {
                let new_kind = match attr.ident.to_string().as_str() {
                    "constructor" => {
                        Ok(ir::FunctionKind::Constructor(ir::KindConstructor {
                            selector: ir::FunctionSelector::from(&method.sig.ident),
                        }))
                    }
                    "message" => {
                        Ok(ir::FunctionKind::Message(ir::KindMessage {
                            selector: ir::FunctionSelector::from(&method.sig.ident),
                        }))
                    }
                    _unknown => Err(format_err!(attr.span(), "unknown ink! marker",)),
                }?;
                if kind == ir::FunctionKind::Method {
                    kind = new_kind;
                    Ok(())
                } else {
                    Err(format_err!(attr.span(), "conflicting ink! marker",))
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
            return Err(format_err_spanned!(
                method.vis,
                "encountered invalid visibility modifier for ink! function",
            ))
        }
        // Functions in ink! must not be `default` since that is unsupported.
        if let Some(defaultness) = method.defaultness {
            return Err(format_err_spanned!(
                defaultness,
                "encountered invalid `default` modifier for ink! function",
            ))
        }
        // Check and convert method signature into ink! function signature.
        let sig = ir::Signature::try_from(method.sig)?;
        // Followed by some checks that are depending on the given function kind:
        match kind {
            ir::FunctionKind::Constructor(_) => {
                if let Some(receiver) = sig.self_arg() {
                    return Err(format_err_spanned!(
                        receiver,
                        "#[ink(constructor)] functions must not have any kind of `self` receiver",
                    ))
                }
                if let syn::ReturnType::Type(_, ty) = &sig.output {
                    let self_ty: syn::Type = syn::parse_quote!(Self);
                    if **ty != self_ty {
                        return Err(format_err_spanned!(
                            sig.output,
                            "#[ink(constructor)] functions must have `Self` return type",
                        ))
                    }
                } else {
                    return Err(format_err_spanned!(
                        sig.output,
                        "#[ink(constructor)] functions must have `Self` return type",
                    ))
                }
            }
            ir::FunctionKind::Message(_) | ir::FunctionKind::Method => {
                if let syn::ReturnType::Type(_, ty) = &sig.output {
                    let self_ty: syn::Type = syn::parse_quote!(Self);
                    if **ty == self_ty {
                        return Err(format_err_spanned!(
                            sig.output,
                            "ink! messages and methods must not return `Self`",
                        ))
                    }
                }
                match sig.self_arg() {
                    Some(receiver) => {
                        if receiver.reference.is_none() {
                            return Err(format_err!(
                                receiver.span(),
                                "ink! messages and methods must have a `&self` or `&mut self` receiver",
                            ))
                        }
                    }
                    None => {
                        return Err(format_err!(
                            sig.span(),
                            "ink! messages and methods must have a `self` receiver",
                        ))
                    }
                }
            }
        }
        // Retain non-ink! attributes only.
        let non_ink_attrs = method
            .attrs
            .into_iter()
            .filter(|attr| ir::Marker::try_from(attr.clone()).is_err())
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

impl TryFrom<syn::Signature> for ir::Signature {
    type Error = syn::Error;

    fn try_from(sig: syn::Signature) -> Result<Self> {
        if let Some(constness) = sig.constness {
            return Err(format_err_spanned!(
                constness,
                "`const fn` is not supported for ink! functions",
            ))
        }
        if let Some(asyncness) = sig.asyncness {
            return Err(format_err_spanned!(
                asyncness,
                "`async fn` is not supported for ink! functions",
            ))
        }
        if let Some(unsafety) = sig.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "`unsafe fn` is not supported for ink! functions",
            ))
        }
        if let Some(abi) = sig.abi {
            return Err(format_err_spanned!(
                abi,
                "specifying ABI is not allowed in ink! functions",
            ))
        }
        if let Some(variadic) = sig.variadic {
            return Err(format_err_spanned!(
                variadic,
                "variadic functions are not allowed as ink! functions",
            ))
        }
        let inputs = sig
            .inputs
            .iter()
            .cloned()
            .map(ir::FnArg::try_from)
            .collect::<Result<Punctuated<ir::FnArg, Token![,]>>>()?;
        Ok(ir::Signature {
            fn_token: sig.fn_token,
            ident: sig.ident,
            generics: sig.generics,
            paren_token: sig.paren_token,
            inputs,
            output: sig.output,
        })
    }
}

impl TryFrom<syn::FnArg> for ir::FnArg {
    type Error = syn::Error;

    fn try_from(fn_arg: syn::FnArg) -> Result<Self> {
        match fn_arg {
            syn::FnArg::Receiver(receiver) => Ok(ir::FnArg::Receiver(receiver)),
            syn::FnArg::Typed(pat_type) => {
                match *pat_type.pat {
                    syn::Pat::Ident(pat_ident) => {
                        if let Some(by_ref) = pat_ident.by_ref {
                            return Err(format_err_spanned!(
                            by_ref,
                            "`ref` modifier is unsupported for ink! function arguments",
                        ))
                        }
                        Ok(ir::FnArg::Typed(ir::IdentType {
                            attrs: pat_ident.attrs,
                            ident: pat_ident.ident,
                            colon_token: pat_type.colon_token,
                            ty: *pat_type.ty,
                        }))
                    }
                    unsupported => {
                        Err(format_err_spanned!(
                    unsupported,
                    "encountered unsupported function argument syntax for ink! function",
                ))
                    }
                }
            }
        }
    }
}

impl TryFrom<syn::Item> for ir::Item {
    type Error = syn::Error;

    fn try_from(item: syn::Item) -> Result<Self> {
        match item {
            syn::Item::Impl(item_impl) => {
                // An ink! `impl` block is identified by having `#[ink(..)]`
                // markers on at least one function or having an `#[ink(impl)]`
                // marker on itself.
                let has_ink_marker_on_impl = utils::has_ink_attributes(&item_impl.attrs);
                // Queries all methods in the `impl` block for `#[ink(..)]` markers.
                let has_ink_marker_on_fns = item_impl
                    .items
                    .iter()
                    .filter_map(|impl_item| {
                        match impl_item {
                            syn::ImplItem::Method(method) => Some(method),
                            _ => None,
                        }
                    })
                    .filter(|method| utils::has_ink_attributes(&method.attrs))
                    .count()
                    > 0;
                if has_ink_marker_on_impl || has_ink_marker_on_fns {
                    ir::ItemImpl::try_from(item_impl)
                        .map(Into::into)
                        .map(ir::Item::Ink)
                } else {
                    Ok(ir::RustItem::from(syn::Item::Impl(item_impl)).into())
                }
            }
            syn::Item::Struct(item_struct) => {
                let markers = utils::filter_map_ink_attributes(&item_struct.attrs)
                    .collect::<Vec<_>>();
                if markers.is_empty() {
                    return Ok(ir::RustItem::from(syn::Item::Struct(item_struct)).into())
                }
                let event_marker =
                    markers.iter().position(|marker| marker.is_simple("event"));
                let storage_marker = markers
                    .iter()
                    .position(|marker| marker.is_simple("storage"));

                match (storage_marker, event_marker) {
                    (Some(_storage_marker), None) => {
                        ir::ItemStorage::try_from(item_struct)
                            .map(Into::into)
                            .map(ir::Item::Ink)
                    }
                    (None, Some(_event_marker)) => {
                        ir::ItemEvent::try_from(item_struct)
                            .map(Into::into)
                            .map(ir::Item::Ink)
                    }
                    (None, None) => {
                        Err(markers
                            .iter()
                            .map(|marker| {
                                format_err!(
                                    marker.span(),
                                    "unsupported ink! marker for struct"
                                )
                            })
                            .fold(
                                format_err_spanned!(
                                    item_struct,
                                    "encountered unsupported ink! markers for struct",
                                ),
                                |mut err1, err2| {
                                    err1.combine(err2);
                                    err1
                                },
                            ))
                    }
                    (Some(storage_marker), Some(event_marker)) => {
                        // Special case: We have both #[ink(storage)] and #[ink(event)].
                        //               This is treated as error but depending on the
                        //               order in which the markers have been provided
                        //               we either treat it as storage or event definition.
                        //
                        // We take whatever ink! marker was provided first.
                        if storage_marker < event_marker {
                            ir::ItemStorage::try_from(item_struct)
                                .map(Into::into)
                                .map(ir::Item::Ink)
                        } else {
                            ir::ItemEvent::try_from(item_struct)
                                .map(Into::into)
                                .map(ir::Item::Ink)
                        }
                    }
                }
            }
            rust_item => Ok(ir::Item::Rust(rust_item.into())),
        }
    }
}

/// Split the storage, the events and functions out of the general contract items vector.
///
/// # Erros
///
/// - When there is not exactly one storage struct.
/// - When a contract item is invalid.
fn split_items(
    items: Vec<ir::InkItem>,
) -> Result<(ir::ItemStorage, Vec<ir::ItemEvent>, Vec<ir::Function>)> {
    let (mut storages, non_storage_items): (Vec<ir::ItemStorage>, Vec<ir::InkItem>) =
        items.into_iter().partition_map(|item| {
            match item {
                ir::InkItem::Storage(item_storage) => Either::Left(item_storage),
                other => Either::Right(other),
            }
        });
    let storage = match storages.len() {
        0 => {
            Err(format_err!(
                Span::call_site(),
                "no #[ink(storage)] struct found but expected exactly 1"
            ))
        }
        1 => {
            Ok(storages
                .pop()
                .expect("there must be exactly one storage in `storages`"))
        }
        n => {
            Err(storages
                .iter()
                .map(|storage| {
                    format_err_spanned!(storage.ident, "conflicting storage struct")
                })
                .fold(
                    format_err!(
                        Span::call_site(),
                        "encountered {} conflicting storage structs",
                        n
                    ),
                    |mut err1, err2| {
                        err1.combine(err2);
                        err1
                    },
                ))
        }
    }?;
    let (events, impl_blocks): (Vec<ir::ItemEvent>, Vec<ir::ItemImpl>) =
        non_storage_items.into_iter().partition_map(|item| {
            match item {
                ir::InkItem::Event(item_event) => Either::Left(item_event),
                ir::InkItem::Impl(item_impl) => Either::Right(item_impl),
                ir::InkItem::Storage(_) => {
                    unreachable!(
                        "we should not have any storages left at this point; qed"
                    )
                }
            }
        });
    let storage_ident = &storage.ident;
    for item_impl in &impl_blocks {
        if &item_impl.self_ty != storage_ident {
            return Err(format_err_spanned!(
                item_impl.self_ty,
                "ink! impl blocks need to be implemented for the #[ink(storage)] struct"
            ))
        }
    }
    let functions = impl_blocks
        .into_iter()
        .map(|impl_block| impl_block.functions)
        .flatten()
        .collect::<Vec<_>>();
    Ok((storage, events, functions))
}
