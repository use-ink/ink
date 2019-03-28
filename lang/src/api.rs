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
        Errors,
        Result,
    },
    hir,
    ident_ext::IdentExt,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::convert::TryFrom;

/// Describes a message parameter or return type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeDescription {
    /// The `bool` primitive type.
    Primitive(PrimitiveTypeDescription),
    /// The tuple type
    Tuple(TupleTypeDescription),
    /// The fixed size array type
    Array(ArrayTypeDescription),
}

impl TryFrom<&syn::Type> for TypeDescription {
    type Error = Errors;

    fn try_from(ty: &syn::Type) -> Result<Self> {
        match ty {
            syn::Type::Tuple(tuple) =>
                TupleTypeDescription::try_from(tuple).map(TypeDescription::Tuple),
            syn::Type::Array(array) =>
                ArrayTypeDescription::try_from(array).map(TypeDescription::Array),
            ty =>
                PrimitiveTypeDescription::try_from(ty).map(TypeDescription::Primitive),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum PrimitiveTypeDescription {
    /// The `bool` primitive type.
    #[serde(rename = "bool")]
    Bool,
    /// The `u8` primitive unsigned integer.
    #[serde(rename = "u8")]
    U8,
    /// The `u16` primitive unsigned integer.
    #[serde(rename = "u16")]
    U16,
    /// The `u32` primitive unsigned integer.
    #[serde(rename = "u32")]
    U32,
    /// The `u64` primitive unsigned integer.
    #[serde(rename = "u64")]
    U64,
    /// The `u128` primitive unsigned integer.
    #[serde(rename = "u128")]
    U128,
    /// The `i8` primitive signed integer.
    #[serde(rename = "i8")]
    I8,
    /// The `i16` primitive signed integer.
    #[serde(rename = "i16")]
    I16,
    /// The `i32` primitive signed integer.
    #[serde(rename = "i32")]
    I32,
    /// The `i64` primitive signed integer.
    #[serde(rename = "i64")]
    I64,
    /// The `i128` primitive signed integer.
    #[serde(rename = "i128")]
    I128,
    /// The SRML address type.
    Address,
    /// The SRML balance type.
    Balance,
}

impl TryFrom<&syn::Type> for PrimitiveTypeDescription {
    type Error = Errors;

    fn try_from(ty: &syn::Type) -> Result<Self> {
        use quote::ToTokens;

        match ty.into_token_stream().to_string().as_str() {
            "bool" => Ok(PrimitiveTypeDescription::Bool),
            "u8" => Ok(PrimitiveTypeDescription::U8),
            "u16" => Ok(PrimitiveTypeDescription::U16),
            "u32" => Ok(PrimitiveTypeDescription::U32),
            "u64" => Ok(PrimitiveTypeDescription::U64),
            "u128" => Ok(PrimitiveTypeDescription::U128),
            "i8" => Ok(PrimitiveTypeDescription::I8),
            "i16" => Ok(PrimitiveTypeDescription::I16),
            "i32" => Ok(PrimitiveTypeDescription::I32),
            "i64" => Ok(PrimitiveTypeDescription::I64),
            "i128" => Ok(PrimitiveTypeDescription::I128),
            "Address" => Ok(PrimitiveTypeDescription::Address),
            "Balance" => Ok(PrimitiveTypeDescription::Balance),
            unsupported => {
                bail!(
                    ty,
                    "{} is unsupported as message interface type",
                    unsupported
                )
            }
        }
    }
}

/// Describes a tuple type
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct TupleTypeDescription {
    elems: Vec<TypeDescription>,
}

impl TryFrom<&syn::TypeTuple> for TupleTypeDescription {
    type Error = Errors;

    fn try_from(arg: &syn::TypeTuple) -> Result<Self> {
        let elems = arg
            .elems
            .iter()
            .map(TypeDescription::try_from)
            .collect::<Result<_>>()?;
        Ok(TupleTypeDescription { elems })
    }
}

/// Describes a fixed size array type
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ArrayTypeDescription {
    #[serde(rename = "[T;n]")]
    FixedLength {
        #[serde(rename = "T")]
        inner: Box<TypeDescription>,
        #[serde(rename = "n")]
        arity: u32
    },
}

impl TryFrom<&syn::TypeArray> for ArrayTypeDescription {
    type Error = Errors;

    fn try_from(arg: &syn::TypeArray) -> Result<Self> {
        let ty = TypeDescription::try_from(&*arg.elem)?;
        if let syn::Expr::Lit(syn::ExprLit {lit: syn::Lit::Int(ref int_lit), .. }) = arg.len {
            Ok(ArrayTypeDescription::FixedLength {
                inner: Box::new(ty),
                arity: int_lit.value() as u32,
            })
        } else {
            bail!(
                arg.len,
                "invalid array length expression"
            )
        }
    }
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ParamDescription {
    /// The name of the parameter.
    name: String,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeDescription,
}

impl TryFrom<&syn::ArgCaptured> for ParamDescription {
    type Error = Errors;

    fn try_from(arg: &syn::ArgCaptured) -> Result<Self> {
        let name = match &arg.pat {
            syn::Pat::Ident(ident) => ident.ident.to_owned_string(),
            _ => {
                bail!(arg.pat, "unsupported type pattern, currently only identifiers like `foo` are supported")
            }
        };
        Ok(Self {
            name,
            ty: TypeDescription::try_from(&arg.ty)?,
        })
    }
}

/// Describes the deploy handler of a contract.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DeployDescription {
    /// The parameters of the deploy handler.
    args: Vec<ParamDescription>,
}

impl TryFrom<&hir::DeployHandler> for DeployDescription {
    type Error = Errors;

    fn try_from(deploy_handler: &hir::DeployHandler) -> Result<Self> {
        let args = deploy_handler
            .decl
            .inputs
            .iter()
            .filter_map(|arg| {
                match arg {
                    ast::FnArg::Captured(captured) => {
                        let description = ParamDescription::try_from(captured);
                        Some(description)
                    }
                    _ => None,
                }
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { args })
    }
}

/// Describes the return type of a contract message.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ReturnTypeDescription {
    #[serde(rename = "type")]
    opt_type: Option<TypeDescription>,
}

impl ReturnTypeDescription {
    /// Creates a new return type description from the given optional type.
    pub fn new<T>(opt_type: T) -> Self
    where
        T: Into<Option<TypeDescription>>,
    {
        Self {
            opt_type: opt_type.into(),
        }
    }
}

impl TryFrom<&syn::ReturnType> for ReturnTypeDescription {
    type Error = Errors;

    fn try_from(ret_ty: &syn::ReturnType) -> Result<Self> {
        match ret_ty {
            syn::ReturnType::Default => Ok(ReturnTypeDescription::new(None)),
            syn::ReturnType::Type(_, ty) => {
                Ok(ReturnTypeDescription::new(Some(TypeDescription::try_from(
                    &**ty,
                )?)))
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
    args: Vec<ParamDescription>,
    /// The return type of the message.
    return_type: ReturnTypeDescription,
}

impl TryFrom<&hir::Message> for MessageDescription {
    type Error = Errors;

    fn try_from(message: &hir::Message) -> Result<Self> {
        Ok(Self {
            name: message.sig.ident.to_owned_string(),
            selector: message.selector().into(),
            mutates: message.is_mut(),
            args: {
                message
                    .sig
                    .decl
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        match arg {
                            ast::FnArg::Captured(captured) => {
                                Some(ParamDescription::try_from(captured))
                            }
                            _ => None,
                        }
                    })
                    .collect::<Result<Vec<_>>>()?
            },
            return_type: ReturnTypeDescription::try_from(&message.sig.decl.output)?,
        })
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

impl TryFrom<&hir::Contract> for ContractDescription {
    type Error = Errors;

    fn try_from(contract: &hir::Contract) -> Result<Self> {
        Ok(ContractDescription {
            name: contract.name.to_owned_string(),
            deploy: DeployDescription::try_from(&contract.on_deploy)?,
            messages: {
                contract
                    .messages
                    .iter()
                    .map(MessageDescription::try_from)
                    .collect::<Result<Vec<_>>>()?
            },
        })
    }
}

/// Writes a JSON API description into the `target/` folder.
pub fn generate_api_description(contract: &hir::Contract) -> Result<()> {
    let description = ContractDescription::try_from(contract)?;
    let contents = serde_json::to_string(&description)
        .expect("Failed at generating JSON API description as JSON");
    let mut path_buf = String::from("target/");
    path_buf.push_str(description.name());
    path_buf.push_str(".json");
    std::fs::create_dir("target");
    std::fs::write(path_buf, contents)
        .expect("Failed at writing JSON API descrition to file");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{*, TypeDescription::*, PrimitiveTypeDescription::*};
    use syn::parse_quote;

    fn assert_eq_type_description(ty: syn::Type, expected: TypeDescription) {
        let actual = TypeDescription::try_from(&ty).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn tuple_basic() {
        assert_eq_type_description(
            parse_quote!( (bool, i32) ),
            Tuple(TupleTypeDescription {
                elems: vec![
                    Primitive(Bool),
                    Primitive(I32),
                ]
            })
        )
    }

    #[test]
    fn tuple_nested() {
        assert_eq_type_description(
            parse_quote!( (u32, (bool, i32)) ),
            Tuple(TupleTypeDescription {
                elems: vec! [
                    Primitive(U32),
                    Tuple(TupleTypeDescription {
                        elems: vec![
                            Primitive(Bool),
                            Primitive(I32),
                        ]
                    }),
                ]
            })
        )
    }

    #[test]
    fn tuple_of_arrays() {
        assert_eq_type_description(
            parse_quote!( ([i32; 2], [u32; 2]) ),
            Tuple(TupleTypeDescription {
                elems: vec! [
                    Array(ArrayTypeDescription::FixedLength {
                        inner: Box::new(Primitive(I32)),
                        arity: 2
                    }),
                    Array(ArrayTypeDescription::FixedLength {
                        inner: Box::new(Primitive(U32)),
                        arity: 2
                    })
                ]
            })
        )
    }

    #[test]
    fn array_basic() {
        assert_eq_type_description(
            parse_quote!( [u32; 5] ),
            Array(ArrayTypeDescription::FixedLength {
                inner: Box::new(Primitive(U32)),
                arity: 5
            })
        )
    }

    #[test]
    fn array_nested() {
        assert_eq_type_description(
            parse_quote!( [[u32; 5]; 3] ),
            Array(ArrayTypeDescription::FixedLength {
                inner: Box::new(Array(ArrayTypeDescription::FixedLength {
                    inner: Box::new(Primitive(U32)),
                    arity: 5
                })),
                arity: 3
            })
        )
    }

    #[test]
    fn array_of_tuples() {
        assert_eq_type_description(
            parse_quote!( [(bool, u32); 5] ),
            Array(ArrayTypeDescription::FixedLength {
                inner: Box::new(Tuple(TupleTypeDescription {
                    elems: vec! [
                        Primitive(Bool),
                        Primitive(U32),
                    ]
                })),
                arity: 5
            })
        )
    }

    #[test]
    fn tuple_json() {
        let ty: syn::Type = parse_quote!( (u64, i32) );
        let td = TypeDescription::try_from(&ty).unwrap();
        let json = serde_json::to_string(&td).unwrap();
        assert_eq!(r#"["u64","i32"]"#, json);
    }

    #[test]
    fn array_json() {
        let ty: syn::Type = parse_quote!( [u32; 5] );
        let td = TypeDescription::try_from(&ty).unwrap();
        let json = serde_json::to_string(&td).unwrap();
        assert_eq!(r#"{"[T;n]":{"T":"u32","n":5}}"#, json);
    }
}

