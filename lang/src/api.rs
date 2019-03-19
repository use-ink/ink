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
    hir,
    ident_ext::IdentExt,
};
use serde::{
    Deserialize,
    Serialize,
};

/// Describes a message parameter or return type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum TypeDescription {
    /// The `bool` primitive type.
    Bool,
    /// The `u8` primitive unsigned integer.
    U8,
    /// The `u16` primitive unsigned integer.
    U16,
    /// The `u32` primitive unsigned integer.
    U32,
    /// The `u64` primitive unsigned integer.
    U64,
    /// The `u128` primitive unsigned integer.
    U128,
    /// The `i8` primitive signed integer.
    I8,
    /// The `i16` primitive signed integer.
    I16,
    /// The `i32` primitive signed integer.
    I32,
    /// The `i64` primitive signed integer.
    I64,
    /// The `i128` primitive signed integer.
    I128,
    /// The SRML address type.
    Address,
    /// The SRML balance type.
    Balance,
    /// Custom type.
    Custom(String),
}

impl From<&syn::Type> for TypeDescription {
    fn from(ty: &syn::Type) -> Self {
        use quote::ToTokens;
        match ty.into_token_stream().to_string().as_str() {
            "bool" => TypeDescription::Bool,
            "u8" => TypeDescription::U8,
            "u16" => TypeDescription::U16,
            "u32" => TypeDescription::U32,
            "u64" => TypeDescription::U64,
            "u128" => TypeDescription::U128,
            "i8" => TypeDescription::I8,
            "i16" => TypeDescription::I16,
            "i32" => TypeDescription::I32,
            "i64" => TypeDescription::I64,
            "i128" => TypeDescription::I128,
            "Address" => TypeDescription::Address,
            "Balance" => TypeDescription::Balance,
            custom => TypeDescription::Custom(custom.to_owned()),
        }
    }
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ParamDescription {
    /// The name of the parameter.
    name: String,
    /// The type of the parameter.
    ty: TypeDescription,
}

impl From<&syn::ArgCaptured> for ParamDescription {
    fn from(arg: &syn::ArgCaptured) -> Self {
        let name = match &arg.pat {
            syn::Pat::Ident(ident) => ident.ident.to_owned_string(),
            _ => panic!("cannot handle non-ident function arguments"),
        };
        Self {
            name,
            ty: TypeDescription::from(&arg.ty),
        }
    }
}

/// Describes the deploy handler of a contract.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DeployDescription {
    /// The parameters of the deploy handler.
    params: Vec<ParamDescription>,
}

impl From<&hir::DeployHandler> for DeployDescription {
    fn from(deploy_handler: &hir::DeployHandler) -> Self {
        Self {
            params: {
                deploy_handler
                    .decl
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        match arg {
                            ast::FnArg::Captured(captured) => {
                                Some(ParamDescription::from(captured))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
            },
        }
    }
}

/// Describes the return type of a contract message.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReturnTypeDescription(Option<TypeDescription>);

impl ReturnTypeDescription {
    /// Creates a new return type description from the given optional type.
    pub fn new<T>(opt_type: T) -> Self
    where
        T: Into<Option<TypeDescription>>,
    {
        Self(opt_type.into())
    }
}

impl From<&syn::ReturnType> for ReturnTypeDescription {
    fn from(ret_ty: &syn::ReturnType) -> Self {
        match ret_ty {
            syn::ReturnType::Default => ReturnTypeDescription::new(None),
            syn::ReturnType::Type(_, ty) => {
                ReturnTypeDescription::new(Some(TypeDescription::from(&**ty)))
            }
        }
    }
}

/// Describes a contract message.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct MessageDescription {
    /// The name of the message.
    name: String,
    /// The selector hash of the message.
    selector: u64,
    /// If the message is allowed to mutate the contract state.
    mutates: bool,
    /// The parameters of the message.
    params: Vec<ParamDescription>,
    /// The return type of the message.
    ret_ty: ReturnTypeDescription,
}

impl From<&hir::Message> for MessageDescription {
    fn from(message: &hir::Message) -> Self {
        Self {
            name: message.sig.ident.to_owned_string(),
            selector: message.selector().into(),
            mutates: message.is_mut(),
            params: {
                message
                    .sig
                    .decl
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        match arg {
                            ast::FnArg::Captured(captured) => {
                                Some(ParamDescription::from(captured))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
            },
            ret_ty: ReturnTypeDescription::from(&message.sig.decl.output),
        }
    }
}

/// Describes a contract.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContractDescription {
    /// The name of the contract.
    name: String,
    /// The deploy handler of the contract.
    deploy: DeployDescription,
    /// The external messages of the contract.
    messages: Vec<MessageDescription>,
}

impl ContractDescription {
    /// Returns the name of the contract.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<&hir::Contract> for ContractDescription {
    fn from(contract: &hir::Contract) -> Self {
        ContractDescription {
            name: contract.name.to_owned_string(),
            deploy: DeployDescription::from(&contract.on_deploy),
            messages: {
                contract
                    .messages
                    .iter()
                    .map(MessageDescription::from)
                    .collect::<Vec<_>>()
            },
        }
    }
}

/// Writes a JSON API description into the `target/` folder.
pub fn generate_api_description(contract: &hir::Contract) {
    let description = ContractDescription::from(contract);
    let contents = serde_json::to_string(&description)
        .expect("Failed at generating JSON API description as JSON");
    let mut path_buf = String::from("target/");
    path_buf.push_str(description.name());
    path_buf.push_str(".json");
    std::fs::write(path_buf, contents)
        .expect("Failed at writing JSON API descrition to file");
}
