// Copyright (C) Use Ink (UK) Ltd.
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

/// Returns the "default" ABI.
///
/// # Note
///
/// The "default" ABI is "ink" unless the ABI is specifically set to "sol".
macro_rules! default_abi {
    () => {{
        if cfg!(ink_abi = "sol") {
            quote!(::ink::reflect::SolEncoding)
        } else {
            quote!(::ink::reflect::ScaleEncoding)
        }
    }};
}

/// Calls the given callback function once for each enabled ABI.
///
/// # Note
///
/// The ABI is passed to the callback function as an argument.
/// The argument value can be either as an `ink_primitives::reflect::Encoding` variant,
/// or tokens for `::ink::reflect::ScaleEncoding` or `::ink::reflect::SolEncoding`.
macro_rules! for_each_abi {
    ($callback: expr, $ink_abi: expr, $sol_abi: expr) => {{
        #[cfg(not(ink_abi = "sol"))]
        $callback($ink_abi);

        #[cfg(any(ink_abi = "sol", ink_abi = "all"))]
        $callback($sol_abi);
    }};
    (@tokens $callback: expr) => {
        for_each_abi!(
            $callback,
            quote!(::ink::reflect::ScaleEncoding),
            quote!(::ink::reflect::SolEncoding)
        )
    };
    (@type $callback: expr) => {
        for_each_abi!(
            $callback,
            ink_primitives::reflect::Encoding::Scale,
            ink_primitives::reflect::Encoding::Solidity
        )
    };
}

/// Generates code for all enabled ABIs by calling the given generator function for each
/// enabled ABI, and returns a `TokenStream` combining all generated ABI specific code.
/// with the ABI as an argument.
///
/// # Note
///
/// The ABI is passed to the generator function as an argument.
/// The argument value can be either as an `ink_primitives::reflect::Encoding` variant,
/// or tokens for `::ink::reflect::ScaleEncoding` or `::ink::reflect::SolEncoding`.
macro_rules! generate_abi_impls {
    ($generator: expr, $ink_abi: expr, $sol_abi: expr) => {{
        let mut abi_impls = Vec::new();
        for_each_abi!(@type |abi| {
            match abi {
                ink_primitives::reflect::Encoding::Scale => {
                    abi_impls.push($generator($ink_abi));
                },
                ink_primitives::reflect::Encoding::Solidity => {
                    abi_impls.push($generator($sol_abi));
                },
            }
        });
        quote! {
            #( #abi_impls )*
        }
    }};
    (@tokens $callback: expr) => {
        generate_abi_impls!(
            $callback,
            quote!(::ink::reflect::ScaleEncoding),
            quote!(::ink::reflect::SolEncoding)
        )
    };
    (@type $callback: expr) => {
        generate_abi_impls!(
            $callback,
            ink_primitives::reflect::Encoding::Scale,
            ink_primitives::reflect::Encoding::Solidity
        )
    };
}
