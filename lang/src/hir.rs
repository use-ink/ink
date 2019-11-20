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

use crate::ast;
use proc_macro2::{
    Ident,
    Span,
};
use syn::{
    self,
    punctuated::Punctuated,
    Result,
    Token,
    Type,
};

/// A smart contract.
#[derive(Debug)]
pub struct Contract {
    /// The name of the smart contract.
    pub name: Ident,
    /// The type of the environment types
    pub env_types_type: Type,
    /// The storage state fields.
    pub state: State,
    /// The deploy handler of the smart contract.
    pub on_deploy: DeployHandler,
    /// The messages of the smart contract.
    pub messages: Vec<Message>,
    /// Methods of the smart contract.
    pub methods: Vec<Method>,
    /// Events of the smart contract.
    pub events: Vec<Event>,
}

/// An event definition.
#[derive(Debug)]
pub struct Event {
    pub attrs: Vec<syn::Attribute>,
    pub ident: Ident,
    pub args: Punctuated<ast::EventArg, Token![,]>,
}

/// Returns an iterator over all doc attributes.
pub fn filter_doc_attributes(
    attrs: &[syn::Attribute],
) -> impl Iterator<Item = &syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.style == syn::AttrStyle::Outer && attr.path.is_ident("doc"))
}

impl Event {
    /// Returns all doc attributes of the message.
    pub fn docs(&self) -> impl Iterator<Item = &syn::Attribute> {
        filter_doc_attributes(&self.attrs)
    }
}

impl Contract {
    /// Extracts the type for environment types from the contract items
    /// and performs some integrity checks on it.
    ///
    /// # Errors
    ///
    /// - If no type for environment types has been found.
    /// - If more than one type for environment types has been found.
    fn extract_env_types(contract: &ast::Contract) -> Result<Type> {
        let env_types = contract
            .env_metas()
            .flat_map(|meta| meta.env_types_metas.clone())
            .map(|meta| {
                if meta.ident == "env" {
                    Ok(meta.ty)
                } else {
                    Err(syn::Error::new(
                        Span::call_site(),
                        format!("unknown env attribute '{}'", meta.ident),
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        if env_types.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "couldn't find an `#![env = <EnvTypesImpl>]` attribute",
            ))
        }
        if env_types.len() > 1 {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "requires exactly one `#![env = <EnvTypesImpl>]` attribute; found {:?}",
                    env_types.len()
                ),
            ))
        }
        Ok(env_types[0].clone())
    }

    /// Extracts all events from the contract.
    ///
    /// Performs some semantic checks on them as a whole.
    ///
    /// # Errors
    ///
    /// - If there are multiple events with the same names.
    /// - If an event has the same name as the contract
    fn extract_events(
        contract_ident: &Ident,
        contract: &ast::Contract,
    ) -> Result<Vec<Event>> {
        let events = contract.events().collect::<Vec<_>>();
        let mut unique_events = std::collections::HashSet::<&ast::ItemEvent>::new();
        for event in &events {
            if &event.ident == contract_ident {
                bail!(
                    event.ident,
                    "cannot declare an event with the same name as the contract",
                );
            }
            if !unique_events.contains(event) {
                unique_events.insert(event);
            } else {
                bail!(
                    event.ident,
                    "cannot declare multiple events with the same name",
                );
            }
        }
        let mut ret = unique_events
            .iter()
            .map(|event| {
                Event {
                    attrs: event.attrs.clone(),
                    ident: event.ident.clone(),
                    args: event.args.clone(),
                }
            })
            .collect::<Vec<_>>();
        ret.sort_by(|e1, e2| e1.ident.partial_cmp(&e2.ident).unwrap());
        Ok(ret)
    }

    /// Extracts the contract state from the contract items
    /// and performs some integrity checks on it.
    ///
    /// # Errors
    ///
    /// - If no contract state has been found.
    /// - If more than one contract state has been found.
    fn extract_state(contract: &ast::Contract) -> Result<(&Ident, State)> {
        let states = contract.states().collect::<Vec<_>>();
        if states.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "couldn't find a contract state `struct`",
            ))
        }
        if states.len() > 1 {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "requires exactly one contract state `struct`; found {:?}",
                    states.len()
                ),
            ))
        }
        let state = states[0];
        Ok((&state.ident, State::from(state)))
    }

    fn unpack_impl_blocks(
        contract_ident: &Ident,
        contract: &ast::Contract,
    ) -> Result<(Vec<Message>, Vec<Method>)> {
        let impl_blocks = contract.impl_blocks().collect::<Vec<_>>();
        if impl_blocks.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "requires at least one contract impl block `struct`; found none",
            ))
        }
        for impl_block in impl_blocks.iter() {
            if impl_block.self_ty != *contract_ident {
                bail!(
                    impl_block.self_ty,
                    "contract impl blocks must implement for the contract type: {}",
                    contract_ident
                );
            }
        }
        use itertools::Itertools as _;
        let (messages, methods): (Vec<_>, Vec<_>) = impl_blocks
            .into_iter()
            .flat_map(|impl_block| impl_block.items.iter())
            .partition_map(|msg_or_method| {
                use either::Either;
                if msg_or_method.vis.is_external() {
                    Either::Left(Message::from(msg_or_method))
                } else {
                    Either::Right(Method::from(msg_or_method))
                }
            });
        for method in methods.iter() {
            if method.sig.ident == "deploy" {
                bail!(
                    method.sig.ident,
                    "contract methods must not be named `deploy`",
                );
            }
        }
        for msg in messages.iter() {
            if msg.sig.ident == "deploy" {
                bail!(
                    msg.sig.ident,
                    "contract messages must not be named `deploy`",
                );
            }
            let inputs = &msg.sig.inputs;
            {
                match inputs.first() {
                    None => {
                        bail!(
                            msg.sig.ident,
                            "contract messages must operate on `&self` or `&mut self`",
                        );
                    }
                    Some(self_ty) => {
                        match self_ty {
                            ast::FnArg::Receiver(syn::Receiver {
                                mutability: None,
                                reference: Some(_),
                                ..
                            })
                            | ast::FnArg::Receiver(syn::Receiver {
                                mutability: Some(_),
                                reference: Some(_),
                                ..
                            }) => (),
                            _ => {
                                bail!(
                                    self_ty,
                                    "contract messages must operate on `&self` or `&mut self`",
                                );
                            }
                        }
                    }
                }
            }
            for fn_arg in inputs.iter().skip(1) {
                if let ast::FnArg::Typed(pat_ty) = fn_arg {
                    if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                        if pat_ident.ident == "env" {
                            bail!(
                                pat_ident.ident,
                                "contract messages must not contain an argument called `env`",
                            );
                        }
                    }
                }
            }
            if msg.sig.generics != Default::default() {
                bail!(msg.sig.generics, "contract messages must not be generic");
            }
        }
        Ok((messages, methods))
    }

    /// Extracts the deploy handler for the given contract identifier
    /// out of the given AST based smart contract token tree.
    ///
    /// # Errors
    ///
    /// - If there is no deploy handler.
    /// - If there is more than one deploy handler for the same contract.
    /// - If there is a deploy handler for a contract that does not exist.
    fn extract_deploy_handler(
        contract_ident: &Ident,
        contract: &ast::Contract,
    ) -> Result<DeployHandler> {
        let mut deploy_impl_blocks: Vec<&ast::ItemDeployImpl> =
            contract.deploy_impl_blocks().collect();
        if deploy_impl_blocks.is_empty() {
            bail!(
                contract_ident,
                "couldn't find a contract deploy implementation; requires exactly one",
            );
        }
        deploy_impl_blocks.retain(|block| block.self_ty == *contract_ident);
        if deploy_impl_blocks.is_empty() {
            bail!(
                contract_ident,
                "couldn't find a contract deploy implementation: `impl Deploy for {} {{ ... }}`",
                contract_ident,
            );
        }
        if deploy_impl_blocks.len() >= 2 {
            bail!(
                contract_ident,
                "found more than one contract deploy implementation for {}",
                contract_ident
            );
        }
        let deploy_impl_block = deploy_impl_blocks[0];

        let sig = &deploy_impl_block.item.sig;
        let self_ty = sig.inputs.first().unwrap();
        match self_ty {
            ast::FnArg::Receiver(syn::Receiver {
                mutability: Some(_),
                reference: Some(_),
                ..
            }) => (),
            _ => {
                bail!(
                    self_ty,
                    "the deploy implementation must operate on `&mut self`",
                );
            }
        }

        for fn_arg in sig.inputs.iter().skip(1) {
            if let ast::FnArg::Typed(pat_typed) = fn_arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_typed.pat {
                    if pat_ident.ident == "env" {
                        bail!(
                            pat_ident.ident,
                            "the deploy implementation must not contain an argument named `env`",
                        );
                    }
                }
            }
        }
        if sig.generics != Default::default() {
            bail!(
                sig.generics,
                "the deploy implementation must not be generic",
            );
        }
        if sig.output != syn::ReturnType::Default {
            bail!(
                sig.output,
                "the deploy implementation must not have a return type",
            );
        }

        Ok(DeployHandler {
            attrs: deploy_impl_block.item.attrs.clone(),
            sig: deploy_impl_block.item.sig.clone(),
            block: deploy_impl_block.item.block.clone(),
        })
    }

    /// Creates a high-level representation contract from the given AST-level contract.
    ///
    /// # Errors
    ///
    /// If any invariant of a contract is invalidated.
    pub fn from_ast(contract: &ast::Contract) -> Result<Self> {
        let env_types_type = Self::extract_env_types(contract)?;
        let (ident, state) = Self::extract_state(contract)?;
        let deploy_handler = Self::extract_deploy_handler(ident, contract)?;
        let (messages, methods) = Self::unpack_impl_blocks(ident, contract)?;
        let events = Self::extract_events(ident, contract)?;
        Ok(Self {
            name: ident.clone(),
            env_types_type,
            state,
            on_deploy: deploy_handler,
            messages,
            methods,
            events,
        })
    }
}

#[derive(Debug)]
pub struct State {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The state fields.
    ///
    /// # Note
    ///
    /// These are the fields that are going to
    /// be stored in the contract storage.
    pub fields: syn::FieldsNamed,
}

impl From<&ast::ItemState> for State {
    fn from(state: &ast::ItemState) -> Self {
        Self {
            attrs: state.attrs.clone(),
            fields: state.fields.clone(),
        }
    }
}

/// The deploy handler of a smart contract.
///
/// # Note
///
/// This is what is getting called upon deploying a smart contract.
/// Normally this is used to initialize storage values.
#[derive(Debug, Clone)]
pub struct DeployHandler {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The function signature.
    pub sig: ast::Signature,
    /// The actual implementation.
    pub block: syn::Block,
}

impl DeployHandler {
    /// Returns all doc attributes of the message.
    pub fn docs(&self) -> impl Iterator<Item = &syn::Attribute> {
        filter_doc_attributes(&self.attrs)
    }

    /// Converts this on-deploy handler into its corresponding message.
    pub fn into_message(self) -> Message {
        Message {
            attrs: self.attrs,
            sig: self.sig,
            block: self.block,
        }
    }
}

impl From<Message> for DeployHandler {
    fn from(msg: Message) -> Self {
        Self {
            attrs: msg.attrs,
            sig: msg.sig,
            block: msg.block,
        }
    }
}

/// A message that is handled by the smart contract.
///
/// # Note
///
/// Messages of a smart contract are only callable externally.
/// They are used to communicate with other smart contracts.
#[derive(Debug)]
pub struct Message {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The message signature.
    ///
    /// # Note
    ///
    /// This also holds the name of the message.
    pub sig: ast::Signature,
    /// The actual implementation.
    pub block: syn::Block,
}

impl Message {
    /// Returns all doc attributes of the message.
    pub fn docs(&self) -> impl Iterator<Item = &syn::Attribute> {
        filter_doc_attributes(&self.attrs)
    }

    /// Returns `true` if the message potentially mutates its state.
    pub fn is_mut(&self) -> bool {
        let self_arg = self
            .sig
            .inputs
            .iter()
            .next()
            .expect("messages must always have at least `&mut self` as parameter");
        match self_arg {
            ast::FnArg::Receiver(syn::Receiver {
                reference,
                mutability,
                ..
            }) => reference.is_some() && mutability.is_some(),
            _ => panic!(),
        }
    }

    /// Returns the message selector for this message.
    pub fn selector(&self) -> [u8; 4] {
        raw_message_selector(self.sig.ident.to_string().as_str())
    }
}

fn raw_message_selector(name: &str) -> [u8; 4] {
    let keccak = ink_utils::hash::keccak256(name.as_bytes());
    [keccak[3], keccak[2], keccak[1], keccak[0]]
}

impl From<&ast::ItemImplMethod> for Message {
    fn from(impl_method: &ast::ItemImplMethod) -> Self {
        Self {
            attrs: impl_method.attrs.clone(),
            sig: impl_method.sig.clone(),
            block: impl_method.block.clone(),
        }
    }
}

/// A method defined on the smart contract.
#[derive(Debug)]
pub struct Method {
    /// The attributes.
    ///
    /// # Note
    ///
    /// Also used for documentation.
    pub attrs: Vec<syn::Attribute>,
    /// The method visibility.
    ///
    /// # Note
    ///
    /// Currently only inherent visibility (private) is
    /// available for methods.
    pub vis: ast::MethodVisibility,
    /// The method signature.
    ///
    /// # Note
    ///
    /// This also holds the name of the method.
    pub sig: ast::Signature,
    /// The actual implementation.
    pub block: syn::Block,
}

impl From<&ast::ItemImplMethod> for Method {
    fn from(impl_method: &ast::ItemImplMethod) -> Self {
        Self {
            attrs: impl_method.attrs.clone(),
            sig: impl_method.sig.clone(),
            vis: impl_method.vis.clone(),
            block: impl_method.block.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_selectors() {
        assert_eq!(raw_message_selector("inc"), [15, 89, 208, 231]);
        assert_eq!(raw_message_selector("get"), [254, 74, 68, 37]);
        assert_eq!(raw_message_selector("compare"), [21, 176, 197, 12]);
    }
}
