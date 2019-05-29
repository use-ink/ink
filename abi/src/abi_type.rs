
pub trait AbiType {}

use crate::TypeSpec;
use serde::{
    Serialize,
    Serializer,
};

macro_rules! impl_serialize_for_primitive_abi_type {
    ( $( $primitive:ty ),* ) => {
        $(
            impl Serialize for TypeSpec<$primitive> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_str(stringify!($primitive))
                }
            }
        )*
    };
}

impl_serialize_for_primitive_abi_type!(
    (), bool,
    i8, i16, i32, i64, i128,
        u16, u32, u64, u128
);

/// Describes how a standard vector shall be serialized.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum VecTypeSpec<T>
where
    T: AbiType,
    TypeSpec<T>: Serialize,
{
    #[serde(rename = "alloc::vec::Vec<T>")]
    Single {
        /// The generic type param.
        #[serde(rename = "T")]
        elem_type: TypeSpec<T>,
    },
}

impl<T> Serialize for TypeSpec<Vec<T>>
where
    T: AbiType,
    TypeSpec<T>: Serialize,
    VecTypeSpec<T>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(
            &VecTypeSpec::<T>::Single { elem_type: TypeSpec::<T>::new() },
            serializer,
        )
    }
}

impl<T> AbiType for Vec<T>
where
    T: AbiType,
{}

/// Describes the option type.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum OptionTypeSpec<T>
where
    T: AbiType,
    TypeSpec<T>: Serialize,
{
    #[serde(rename = "Option<T>")]
    Single {
        /// The generic type param.
        #[serde(rename = "T")]
        inner: TypeSpec<T>,
    },
}

impl<T> AbiType for Option<T>
where
    T: AbiType,
{}

impl<T> Serialize for TypeSpec<Option<T>>
where
    T: AbiType,
    TypeSpec<T>: Serialize,
    OptionTypeSpec<T>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(
            &OptionTypeSpec::<T>::Single { inner: TypeSpec::<T>::new() },
            serializer,
        )
    }
}

/// Describes a result param or return type.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum ResultTypeSpec<Ok, Err>
where
    Ok: AbiType,
    Err: AbiType,
    TypeSpec<Ok>: Serialize,
    TypeSpec<Err>: Serialize,
{
    #[serde(rename = "Result<T,E>")]
    Single {
        /// The `Ok`-type.
        #[serde(rename = "T")]
        ok_type: TypeSpec<Ok>,
        /// The `Err`-type.
        #[serde(rename = "E")]
        err_type: TypeSpec<Err>,
    },
}

impl<Ok, Err> AbiType for Result<Ok, Err>
where
    Ok: AbiType,
    Err: AbiType,
{}

impl<Ok, Err> Serialize for TypeSpec<Result<Ok, Err>>
where
    Ok: AbiType,
    Err: AbiType,
    TypeSpec<Ok>: Serialize,
    TypeSpec<Err>: Serialize,
    ResultTypeSpec<Ok, Err>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(
            &ResultTypeSpec::<Ok, Err>::Single {
                ok_type: TypeSpec::<Ok>::new(),
                err_type: TypeSpec::<Err>::new(),
            },
            serializer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use json::json;

    #[test]
    fn simple() {
        assert_eq!(json::to_value(&TypeSpec::<()>::new()).unwrap(), json!("()"));
        assert_eq!(json::to_value(&TypeSpec::<bool>::new()).unwrap(), json!("bool"));
        assert_eq!(json::to_value(&TypeSpec::<i32>::new()).unwrap(), json!("i32"));
        assert_eq!(json::to_value(&TypeSpec::<u128>::new()).unwrap(), json!("u128"));
    }

    #[test]
    fn option() {
        assert_eq!(
            json::to_value(
                &TypeSpec::<Option<i32>>::new()
            ).unwrap(),
            json!({
                "Option<T>": {
                    "T": "i32"
                }
            })
        );
        assert_eq!(
            json::to_value(
                &TypeSpec::<Option<Option<i32>>>::new()
            ).unwrap(),
            json!({
                "Option<T>": {
                    "T": {
                        "Option<T>": {
                            "T": "i32"
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn result() {
        assert_eq!(
            json::to_value(
                &TypeSpec::<Result<i32, bool>>::new()
            ).unwrap(),
            json!({
                "Result<T,E>": {
                    "T": "i32",
                    "E": "bool"
                }
            })
        );
        assert_eq!(
            json::to_value(
                &TypeSpec::<Result<Result<i32, i8>, bool>>::new()
            ).unwrap(),
            json!({
                "Result<T,E>": {
                    "T": {
                        "Result<T,E>": {
                            "T": "i32",
                            "E": "i8"
                        }
                    },
                    "E": "bool"
                }
            })
        );
    }

    #[test]
    fn vec() {
        assert_eq!(
            json::to_value(&TypeSpec::<Vec<i32>>::new()).unwrap(),
            json!({
                "alloc::vec::Vec<T>": {
                    "T": "i32"
                }
            })
        );
        assert_eq!(
            json::to_value(&TypeSpec::<Vec<Vec<bool>>>::new()).unwrap(),
            json!({
                "alloc::vec::Vec<T>": {
                    "T": {
                        "alloc::vec::Vec<T>": {
                            "T": "bool"
                        }
                    }
                }
            })
        );
    }
}
