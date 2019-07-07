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

use crate::{
    ast,
    hir,
};
use serde::{
    Deserialize,
    Serialize,
};
use syn::{self, Result};
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
    /// A concrete `Option` type.
    Option(OptionTypeDescription),
    /// A concrete `Result` type.
    Result(ResultTypeDescription),
    /// A concrete `Vec` type.
    Vec(VecTypeDescription),
}

/// Describes an option param or return type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum OptionTypeDescription {
    #[serde(rename = "Option<T>")]
    Single {
        /// The generic type param.
        #[serde(rename = "T")]
        inner: Box<TypeDescription>,
    },
}

impl TryFrom<&syn::TypePath> for OptionTypeDescription {
    type Error = syn::Error;

    fn try_from(type_path: &syn::TypePath) -> Result<Self> {
        if type_path.qself.is_some() || type_path.path.leading_colon.is_some() {
            bail!(type_path, "`Option` cannot be qualified or start with `::`")
        }
        if type_path.path.segments.len() != 1 {
            bail!(type_path, "too many path segments for an `Option` type")
        }
        let seg = &type_path.path.segments[0];
        if seg.ident != "Option" {
            bail!(type_path, "invalid ident for `Option` type")
        }
        match &seg.arguments {
            syn::PathArguments::AngleBracketed(generic_args) => {
                if generic_args.args.len() != 1 {
                    bail!(generic_args, "too many generic args for `Option` type")
                }
                match &generic_args.args[0] {
                    syn::GenericArgument::Type(ty) => {
                        Ok(OptionTypeDescription::Single {
                            inner: Box::new(TypeDescription::try_from(ty)?),
                        })
                    }
                    invalid => bail!(invalid, "invalid generic type args for `Option`"),
                }
            }
            invalid => bail!(invalid, "invalid type arguments for `Option`"),
        }
    }
}

/// Describes a `Vec` param or return type.
///
/// # Note
///
/// With `Vec` we refer to `memory::Vec` here.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum VecTypeDescription {
    #[serde(rename = "Vec<T>")]
    Single {
        /// The generic type param.
        #[serde(rename = "T")]
        elem_type: Box<TypeDescription>,
    },
}

impl TryFrom<&syn::TypePath> for VecTypeDescription {
    type Error = syn::Error;

    fn try_from(type_path: &syn::TypePath) -> Result<Self> {
        if type_path.qself.is_some() || type_path.path.leading_colon.is_some() {
            bail!(type_path, "`Vec` cannot be qualified or start with `::`")
        }
        if type_path.path.segments.len() != 1 {
            bail!(type_path, "too many path segments for an `Vec` type")
        }
        let seg = &type_path.path.segments[0];
        if seg.ident != "Vec" {
            bail!(type_path, "invalid ident for `Vec` type")
        }
        match &seg.arguments {
            syn::PathArguments::AngleBracketed(generic_args) => {
                if generic_args.args.len() != 1 {
                    bail!(generic_args, "too many generic args for `Vec` type")
                }
                match &generic_args.args[0] {
                    syn::GenericArgument::Type(ty) => {
                        Ok(VecTypeDescription::Single {
                            elem_type: Box::new(TypeDescription::try_from(ty)?),
                        })
                    }
                    invalid => bail!(invalid, "invalid generic type args for `Vec`"),
                }
            }
            invalid => bail!(invalid, "invalid type arguments for `Vec`"),
        }
    }
}

/// Describes a result param or return type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ResultTypeDescription {
    #[serde(rename = "Result<T,E>")]
    Single {
        /// The `Ok`-type.
        #[serde(rename = "T")]
        ok_type: Box<TypeDescription>,
        /// The `Err`-type.
        #[serde(rename = "E")]
        err_type: Box<TypeDescription>,
    },
}

impl TryFrom<&syn::TypePath> for ResultTypeDescription {
    type Error = syn::Error;

    fn try_from(type_path: &syn::TypePath) -> Result<Self> {
        if type_path.qself.is_some() || type_path.path.leading_colon.is_some() {
            bail!(type_path, "`Result` cannot be qualified or start with `::`")
        }
        if type_path.path.segments.len() != 1 {
            bail!(type_path, "too many path segments for an `Result` type")
        }
        let seg = &type_path.path.segments[0];
        if seg.ident != "Result" {
            bail!(type_path, "invalid ident for `Result` type")
        }
        match &seg.arguments {
            syn::PathArguments::AngleBracketed(generic_args) => {
                if generic_args.args.len() != 2 {
                    bail!(
                        generic_args,
                        "`Result` type requires 2 generic type arguments"
                    )
                }
                let ok_type = match &generic_args.args[0] {
                    syn::GenericArgument::Type(ty) => TypeDescription::try_from(ty),
                    invalid => bail!(invalid, "invalid generic type args for `Result`"),
                }?;
                let err_type = match &generic_args.args[1] {
                    syn::GenericArgument::Type(ty) => TypeDescription::try_from(ty),
                    invalid => bail!(invalid, "invalid generic type args for `Result`"),
                }?;
                Ok(ResultTypeDescription::Single {
                    ok_type: Box::new(ok_type),
                    err_type: Box::new(err_type),
                })
            }
            invalid => bail!(invalid, "invalid type arguments for `Result`"),
        }
    }
}

impl TryFrom<&syn::Type> for TypeDescription {
    type Error = syn::Error;

    fn try_from(ty: &syn::Type) -> Result<Self> {
        match ty {
            syn::Type::Tuple(tuple) => {
                TupleTypeDescription::try_from(tuple).map(TypeDescription::Tuple)
            }
            syn::Type::Array(array) => {
                ArrayTypeDescription::try_from(array).map(TypeDescription::Array)
            }
            syn::Type::Path(path) => {
                if path.path.segments.len() != 1 || path.path.leading_colon.is_some() {
                    bail!(path, "invalid self qualifier or leading `::` for type")
                }
                let ident = &path.path.segments[0].ident;
                match ident.to_string().as_str() {
                    "Option" => {
                        OptionTypeDescription::try_from(path).map(TypeDescription::Option)
                    }
                    "Result" => {
                        ResultTypeDescription::try_from(path).map(TypeDescription::Result)
                    }
                    "Vec" => VecTypeDescription::try_from(path).map(TypeDescription::Vec),
                    _ => {
                        PrimitiveTypeDescription::try_from(path)
                            .map(TypeDescription::Primitive)
                    }
                }
            }
            invalid => bail!(invalid, "invalid or unsupported type",),
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
    AccountId,
    /// The SRML balance type.
    Balance,
    /// The SRML hash type.
    Hash,
    /// The SRML moment type.
    Moment,
    /// The SRML block number type.
    BlockNumber,
}

impl TryFrom<&syn::TypePath> for PrimitiveTypeDescription {
    type Error = syn::Error;

    fn try_from(ty: &syn::TypePath) -> Result<Self> {
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
            "AccountId" => Ok(PrimitiveTypeDescription::AccountId),
            "Balance" => Ok(PrimitiveTypeDescription::Balance),
            "Hash" => Ok(PrimitiveTypeDescription::Hash),
            "Moment" => Ok(PrimitiveTypeDescription::Moment),
            "BlockNumber" => Ok(PrimitiveTypeDescription::BlockNumber),
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
    type Error = syn::Error;

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
        arity: u32,
    },
}

impl TryFrom<&syn::TypeArray> for ArrayTypeDescription {
    type Error = syn::Error;

    fn try_from(arg: &syn::TypeArray) -> Result<Self> {
        let ty = TypeDescription::try_from(&*arg.elem)?;
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(ref int_lit),
            ..
        }) = arg.len
        {
            Ok(ArrayTypeDescription::FixedLength {
                inner: Box::new(ty),
                arity: int_lit.value() as u32,
            })
        } else {
            bail!(arg.len, "invalid array length expression")
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
    type Error = syn::Error;

    fn try_from(arg: &syn::ArgCaptured) -> Result<Self> {
        let name = match &arg.pat {
            syn::Pat::Ident(ident) => ident.ident.to_string(),
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
    type Error = syn::Error;

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
    type Error = syn::Error;

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
    type Error = syn::Error;

    fn try_from(message: &hir::Message) -> Result<Self> {
        Ok(Self {
            name: message.sig.ident.to_string(),
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
    type Error = syn::Error;

    fn try_from(contract: &hir::Contract) -> Result<Self> {
        Ok(ContractDescription {
            name: contract.name.to_string(),
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
    std::fs::create_dir("target").unwrap_or(());
    std::fs::write(path_buf, contents)
        .expect("Failed at writing JSON API descrition to file");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        PrimitiveTypeDescription::*,
        TypeDescription::*,
        *,
    };
    use syn::parse_quote;

    fn assert_eq_type_description(ty: syn::Type, expected: TypeDescription) {
        let actual = TypeDescription::try_from(&ty).unwrap();
        assert_eq!(expected, actual);
    }

    fn assert_json_roundtrip(ty: syn::Type, json: &str) {
        let td = TypeDescription::try_from(&ty).unwrap();
        let actual_json = serde_json::to_string(&td).unwrap();
        assert_eq!(json, actual_json);
        let deserialized: TypeDescription = serde_json::de::from_str(json).unwrap();
        assert_eq!(td, deserialized);
    }

    #[test]
    fn primitives() {
        assert_eq_type_description(
            parse_quote!(u16),
            Primitive(PrimitiveTypeDescription::U16),
        );
        assert_eq_type_description(
            parse_quote!(bool),
            Primitive(PrimitiveTypeDescription::Bool),
        );
        assert_eq_type_description(
            parse_quote!(i64),
            Primitive(PrimitiveTypeDescription::I64),
        );
        assert_eq_type_description(
            parse_quote!(AccountId),
            Primitive(PrimitiveTypeDescription::AccountId),
        );
        assert_eq_type_description(
            parse_quote!(Moment),
            Primitive(PrimitiveTypeDescription::Moment),
        );
        assert_eq_type_description(
            parse_quote!(BlockNumber),
            Primitive(PrimitiveTypeDescription::BlockNumber),
        );
    }

    #[test]
    fn tuple_basic() {
        assert_eq_type_description(
            parse_quote!((bool, i32)),
            Tuple(TupleTypeDescription {
                elems: vec![Primitive(Bool), Primitive(I32)],
            }),
        )
    }

    #[test]
    fn tuple_nested() {
        assert_eq_type_description(
            parse_quote!((u32, (bool, i32))),
            Tuple(TupleTypeDescription {
                elems: vec![
                    Primitive(U32),
                    Tuple(TupleTypeDescription {
                        elems: vec![Primitive(Bool), Primitive(I32)],
                    }),
                ],
            }),
        )
    }

    #[test]
    fn tuple_of_arrays() {
        assert_eq_type_description(
            parse_quote!(([i32; 2], [u32; 2])),
            Tuple(TupleTypeDescription {
                elems: vec![
                    Array(ArrayTypeDescription::FixedLength {
                        inner: Box::new(Primitive(I32)),
                        arity: 2,
                    }),
                    Array(ArrayTypeDescription::FixedLength {
                        inner: Box::new(Primitive(U32)),
                        arity: 2,
                    }),
                ],
            }),
        )
    }

    #[test]
    fn array_basic() {
        assert_eq_type_description(
            parse_quote!([u32; 5]),
            Array(ArrayTypeDescription::FixedLength {
                inner: Box::new(Primitive(U32)),
                arity: 5,
            }),
        )
    }

    #[test]
    fn array_nested() {
        assert_eq_type_description(
            parse_quote!([[u32; 5]; 3]),
            Array(ArrayTypeDescription::FixedLength {
                inner: Box::new(Array(ArrayTypeDescription::FixedLength {
                    inner: Box::new(Primitive(U32)),
                    arity: 5,
                })),
                arity: 3,
            }),
        )
    }

    #[test]
    fn array_of_tuples() {
        assert_eq_type_description(
            parse_quote!([(bool, u32); 5]),
            Array(ArrayTypeDescription::FixedLength {
                inner: Box::new(Tuple(TupleTypeDescription {
                    elems: vec![Primitive(Bool), Primitive(U32)],
                })),
                arity: 5,
            }),
        )
    }

    #[test]
    fn tuple_json() {
        assert_json_roundtrip(parse_quote!((u64, i32)), r#"["u64","i32"]"#)
    }

    #[test]
    fn array_json() {
        assert_json_roundtrip(parse_quote!([u32; 5]), r#"{"[T;n]":{"T":"u32","n":5}}"#)
    }

    fn expect_failure(input: syn::Type, expected_err: &str) {
        let res = TypeDescription::try_from(&input).map_err(|err| format!("{}", err));
        assert_eq!(res, Err(String::from(expected_err)));
    }

    #[test]
    fn option_json_failure() {
        expect_failure(
            parse_quote!(<Self as Foo>::Option<i32>),
            "invalid self qualifier or leading `::` for type",
        );
        expect_failure(
            parse_quote!(::Option<i32>),
            "invalid self qualifier or leading `::` for type",
        );
        expect_failure(
            parse_quote!(Option<bool, i32>),
            "too many generic args for `Option` type",
        );
        expect_failure(
            parse_quote!(Option<'a>),
            "invalid generic type args for `Option`",
        );
    }

    #[test]
    fn option_json_success() {
        assert_json_roundtrip(parse_quote!(Option<i32>), r#"{"Option<T>":{"T":"i32"}}"#);
        assert_json_roundtrip(
            parse_quote!(Option<(bool, i32)>),
            r#"{"Option<T>":{"T":["bool","i32"]}}"#,
        );
        assert_json_roundtrip(
            parse_quote!(Option<Option<i32>>),
            r#"{"Option<T>":{"T":{"Option<T>":{"T":"i32"}}}}"#,
        );
    }

    #[test]
    fn vec_json_failure() {
        expect_failure(
            parse_quote!(<Self as Foo>::Vec<i32>),
            "invalid self qualifier or leading `::` for type",
        );
        expect_failure(
            parse_quote!(::Vec<i32>),
            "invalid self qualifier or leading `::` for type",
        );
        expect_failure(
            parse_quote!(Vec<bool, i32>),
            "too many generic args for `Vec` type",
        );
        expect_failure(parse_quote!(Vec<'a>), "invalid generic type args for `Vec`");
    }

    #[test]
    fn vec_json_success() {
        assert_json_roundtrip(parse_quote!(Vec<i32>), r#"{"Vec<T>":{"T":"i32"}}"#);
        assert_json_roundtrip(
            parse_quote!(Vec<(bool, i32)>),
            r#"{"Vec<T>":{"T":["bool","i32"]}}"#,
        );
        assert_json_roundtrip(
            parse_quote!(Vec<Vec<i32>>),
            r#"{"Vec<T>":{"T":{"Vec<T>":{"T":"i32"}}}}"#,
        );
    }

    #[test]
    fn result_json_failure() {
        expect_failure(
            parse_quote!(<Self as Foo>::Result<bool, i32>),
            "invalid self qualifier or leading `::` for type",
        );
        expect_failure(
            parse_quote!(::Result<bool, i32>),
            "invalid self qualifier or leading `::` for type",
        );
        expect_failure(
            parse_quote!(Result<u32>),
            "`Result` type requires 2 generic type arguments",
        );
        expect_failure(
            parse_quote!(Result<u16, u32, u64>),
            "`Result` type requires 2 generic type arguments",
        );
        expect_failure(
            parse_quote!(Result<'a, bool>),
            "invalid generic type args for `Result`",
        );
    }

    #[test]
    fn result_json_success() {
        assert_json_roundtrip(
            parse_quote!(Result<bool, i32>),
            r#"{"Result<T,E>":{"T":"bool","E":"i32"}}"#,
        );
        assert_json_roundtrip(
            parse_quote!(Result<(bool, i32), [u8; 8]>),
            r#"{"Result<T,E>":{"T":["bool","i32"],"E":{"[T;n]":{"T":"u8","n":8}}}}"#,
        );
        assert_json_roundtrip(
            parse_quote!(Result<Result<u8,i8>,Result<u16,i16>>),
            r#"{"Result<T,E>":{"T":{"Result<T,E>":{"T":"u8","E":"i8"}},"E":{"Result<T,E>":{"T":"u16","E":"i16"}}}}"#,
        );
    }
}
