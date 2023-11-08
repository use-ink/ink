// Copyright (C) Parity Technologies (UK) Ltd.
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

/// Trait implemented by chain extensions.
///
/// Allows to use the `self.env().extension().my_chain_extension(...)` syntax.
///
/// # Note
///
/// This trait is automatically implemented when using `#[ink::chain_extension]`
/// procedural macro.
pub trait ChainExtensionInstance {
    /// The type of the chain extension instance.
    type Instance;

    /// Creates a new instance of the chain extension to use methods with method chaining
    /// syntax.
    fn instantiate() -> Self::Instance;
}

/// Implemented by chain extension types.
///
/// Every chain extension defines a set of chain extension methods
/// that share a common error code type.
pub trait ChainExtension {
    /// The error code that determines whether a chain extension method call was
    /// successful.
    type ErrorCode: ink_env::chain_extension::FromStatusCode;
}

/// Only implemented for `Result<T, E>`.
///
/// Used to check at compile time if the chain extension method return type
/// is a `Result` type using the type system instead of the syntactic structure.
#[doc(hidden)]
pub trait IsResultType: private::IsResultSealed {
    /// The `T` type of the `Result<T, E>`.
    type Ok;
    /// The `E` type of the `Result<T, E>`.
    type Err;
}

impl<T, E> private::IsResultSealed for Result<T, E> {}
impl<T, E> IsResultType for Result<T, E> {
    type Ok = T;
    type Err = E;
}

/// Only implemented for [`ValueReturned`].
///
/// Used to deduce the correct return type of a chain extension method at compile time
/// based on 2 flags: `const IS_RESULT: bool` and `const HANDLE_STATUS: bool`.
///
/// If `IS_RESULT` is set to `false` and `HANDLE_STATUS` is `true`, then
/// `type ReturnType = Result<T, E>`. Otherwise `type ReturnType = T`.
pub trait Output<const IS_RESULT: bool, const HANDLE_STATUS: bool, T, E>:
    private::OutputSealed
{
    type ReturnType;
}

/// Represents some abstract value that is returned by a function.
/// Currently acts as a placeholder.
pub struct ValueReturned;
impl private::OutputSealed for ValueReturned {}

impl<T, E> Output<false, false, T, E> for ValueReturned {
    type ReturnType = T;
}

impl<T, E> Output<false, true, T, E> for ValueReturned {
    type ReturnType = core::result::Result<T, E>;
}

impl<T, E> Output<true, false, T, E> for ValueReturned {
    type ReturnType = T;
}

impl<T, E> Output<true, true, T, E> for ValueReturned {
    type ReturnType = T;
}

mod private {
    /// Seals the `IsResultType` trait so that it cannot be implemented outside this
    /// module.
    pub trait IsResultSealed {}
    /// Seals the `Output` trait so that it cannot be implemented outside this module.
    pub trait OutputSealed {}
}

/// Macro defines the combined chain extension via structure definition.
/// Each sub-extension can be accessed by the corresponding field.
///
/// The macro expects a structure definition as an input where each field should
/// implement [`ChainExtensionInstance`]. Usually, this trait is implemented
/// by the `#[ink::chain_extension]` macro during the definition of the chain extension.
///
/// ```rust
/// #[ink::scale_derive(TypeInfo)]
/// struct ExtensionOne;
/// impl ink::ChainExtensionInstance for ExtensionOne {
///     type Instance = Self;
///
///     fn instantiate() -> Self::Instance {
///         Self {}
///     }
/// }
///
/// #[ink::scale_derive(TypeInfo)]
/// struct ExtensionTwo;
/// impl ink::ChainExtensionInstance for ExtensionTwo {
///     type Instance = Self;
///
///     fn instantiate() -> Self::Instance {
///         Self {}
///     }
/// }
///
/// ink::combine_extensions! {
///     /// Defines a combined extension with [`ExtensionOne`] and [`ExtensionTwo`].
///     struct Combined {
///         /// This field can be used to access the first extension like
///         /// `self.env().extension().extension_one` in the contract's context.
///         extension_one: ExtensionOne,
///         /// This field can be used to access the second extension like
///         /// `self.env().extension().extension_two` in the contract's context.
///         extension_two: ExtensionTwo,
///     }
/// }
/// ```
#[macro_export]
macro_rules! combine_extensions {
    ($(#[$meta:meta])* $vis:vis struct $name:ident {
        $($(#[$field_meta:meta])* $field_vis:vis $field_name:ident: $field_type:ty,)*
    }) => {
        $(#[$meta])*
        #[::ink::scale_derive(TypeInfo)]
        $vis struct $name {
            $($(#[$field_meta])* $field_vis $field_name: $field_type,)*
        }

        const _: () = {
            /// Each chain extension has an abstract type `$name` that describes
            /// it and the actual instance that provides access to methods.
            /// This structure is an instance that is returned by the `self.env().extension()` call.
            ///
            /// Because it is a combination of corresponding sub-instances, we need to initialize
            /// each sub-instance in the same way by calling [`ChainExtensionInstance::instantiate`].
            pub struct Private {
                $($(#[$field_meta])* $field_vis $field_name: <$field_type as ::ink::ChainExtensionInstance>::Instance,)*
            }

            impl ::ink::ChainExtensionInstance for $name {
                type Instance = Private;

                fn instantiate() -> Self::Instance {
                    Self::Instance {
                        $($field_name: <$field_type as ::ink::ChainExtensionInstance>::instantiate(),)*
                    }
                }
            }
        };
    }
}
