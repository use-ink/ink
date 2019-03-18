// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    ast,
    errors::{
        Result,
        SynError,
    },
};
use proc_macro2::{
    Ident,
    Span,
};

/// A smart contract.
#[derive(Debug)]
pub struct Contract {
    /// The name of the smart contract.
    pub name: Ident,
    /// The storage state fields.
    pub state: State,
    /// The deploy handler of the smart contract.
    pub on_deploy: DeployHandler,
    /// The messages of the smart contract.
    pub messages: Vec<Message>,
    /// Methods of the smart contract.
    pub methods: Vec<Method>,
}

impl Contract {
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
            return Err(SynError::new(
                Span::call_site(),
                "couldn't find a contract state `struct`",
            )
            .into())
        }
        if states.len() > 1 {
            return Err(SynError::new(
                Span::call_site(),
                format!(
                    "requires exactly one contract state `struct`; found {:?}",
                    states.len()
                ),
            )
            .into())
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
            return Err(SynError::new(
                Span::call_site(),
                "requires at least one contract impl block `struct`; found none",
            )
            .into())
        }
        for impl_block in impl_blocks.iter() {
            if impl_block.self_ty != *contract_ident {
                bail!(
                    impl_block.self_ty,
                    "contract impl blocks must implement for the contract type: {}",
                    contract_ident
                )
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
                    "contract methods must not be named `deploy`"
                )
            }
        }
        for msg in messages.iter() {
            if msg.sig.ident == "deploy" {
                bail!(
                    msg.sig.ident,
                    "contract messages must not be named `deploy`"
                )
            }
            let inputs = &msg.sig.decl.inputs;
            {
                match inputs.first().map(|arg| arg.into_value()) {
                    None => {
                        bail!(
                            msg.sig.ident,
                            "contract messages must operate on `&self` or `&mut self`"
                        )
                    }
                    Some(self_ty) => match self_ty {
                        ast::FnArg::SelfValue(_) | ast::FnArg::Captured(_) => {
                            bail!(
                                self_ty,
                                "contract messages must operate on `&self` or `&mut self`"
                            )
                        }
                        _ => (),
                    }
                }
            }
            for fn_arg in inputs.iter().skip(1) {
                if let ast::FnArg::Captured(arg_captured) = fn_arg {
                    if let syn::Pat::Ident(pat_ident) = &arg_captured.pat {
                        if pat_ident.ident == "env" {
                            bail!(
                                pat_ident.ident,
                                "contract messages must not contain an argument called `env`"
                            )
                        }
                    }
                }
            }
            if msg.sig.decl.generics != Default::default() {
                bail!(
                    msg.sig.decl.generics,
                    "contract messages must not be generic"
                )
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
        let mut deploy_impl_blocks = contract.deploy_impl_blocks().collect::<Vec<_>>();
        if deploy_impl_blocks.is_empty() {
            bail!(
                contract_ident,
                "couldn't find a contract deploy implementation; requires exactly one",
            )
        }
        deploy_impl_blocks.retain(|block| block.self_ty == *contract_ident);
        if deploy_impl_blocks.is_empty() {
            bail!(
                contract_ident,
                "couldn't find a contract deploy implementation: `impl Deploy for {} {{ ... }}`",
                contract_ident,
            )
        }
        if deploy_impl_blocks.len() >= 2 {
            bail!(
                contract_ident,
                "found more than one contract deploy implementation for {}",
                contract_ident
            )
        }
        let deploy_impl_block = deploy_impl_blocks[0];

        let fn_decl = &deploy_impl_block.item.decl;
        let self_ty: &ast::FnArg = &fn_decl.inputs.first().unwrap().into_value();

        if let ast::FnArg::SelfRef(syn::ArgSelfRef {
            mutability: None, ..
        })
        | ast::FnArg::SelfValue(_)
        | ast::FnArg::Captured(_) = self_ty
        {
            bail!(
                self_ty,
                "the deploy implementation must operate on `&mut self`"
            )
        }

        for fn_arg in fn_decl.inputs.iter().skip(1) {
            if let ast::FnArg::Captured(arg_captured) = fn_arg {
                if let syn::Pat::Ident(pat_ident) = &arg_captured.pat {
                    if pat_ident.ident == "env" {
                        bail!(
                            pat_ident.ident,
                            "the deploy implementation must not contain an argument named `env`"
                        )
                    }
                }
            }
        }
        if fn_decl.generics != Default::default() {
            bail!(
                fn_decl.generics,
                "the deploy implementation must not be generic"
            )
        }
        if fn_decl.output != syn::ReturnType::Default {
            bail!(
                fn_decl.output,
                "the deploy implementation must not have a return type"
            )
        }

        Ok(DeployHandler {
            attrs: deploy_impl_block.item.attrs.clone(),
            decl: deploy_impl_block.item.decl.clone(),
            block: deploy_impl_block.item.block.clone(),
        })
    }

    /// Creates a high-level representation contract from the given AST-level contract.
    ///
    /// # Errors
    ///
    /// If any invariant of a contract is invalidated.
    pub fn from_ast(contract: &ast::Contract) -> Result<Self> {
        let (ident, state) = Self::extract_state(contract)?;
        let deploy_handler = Self::extract_deploy_handler(ident, contract)?;
        let (messages, methods) = Self::unpack_impl_blocks(ident, contract)?;
        Ok(Self {
            name: ident.clone(),
            state,
            on_deploy: deploy_handler,
            messages,
            methods,
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
    /// The function declaration.
    pub decl: ast::FnDecl,
    /// The actual implementation.
    pub block: syn::Block,
}

impl DeployHandler {
    /// Converts this on-deploy handler into its corresponding message.
    pub fn into_message(self) -> Message {
        use crate::ident_ext::IdentExt as _;
        Message {
            attrs: self.attrs,
            sig: ast::MethodSig {
                ident: Ident::from_str("deploy"),
                decl: self.decl,
            },
            block: self.block,
        }
    }
}

impl From<Message> for DeployHandler {
    fn from(msg: Message) -> Self {
        Self {
            attrs: msg.attrs,
            decl: msg.sig.decl,
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
    pub sig: ast::MethodSig,
    /// The actual implementation.
    pub block: syn::Block,
}

impl Message {
    /// Returns `true` if the message potentially mutates its state.
    pub fn is_pub(&self) -> bool {
        let self_arg = self.sig.decl.inputs.iter().next().unwrap();
        match self_arg {
            ast::FnArg::SelfRef(syn::ArgSelfRef { mutability, .. }) => {
                mutability.is_some()
            }
            _ => panic!(),
        }
    }
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
    pub sig: ast::MethodSig,
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
