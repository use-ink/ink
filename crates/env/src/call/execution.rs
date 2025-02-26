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

use super::{
    utils::ReturnType,
    Selector,
};
use crate::Environment;
use alloy_sol_types::{
    private::SolTypeValue,
    SolType,
    SolValue,
    Word,
};
use core::marker::PhantomData;

/// The input data and the expected return type of a contract execution.
pub struct Execution<Args, Output, Strategy> {
    /// The input data for initiating a contract execution.
    pub input: ExecutionInput<Args, Strategy>,
    /// The type of the expected return value of the contract execution.
    pub output: ReturnType<Output>,
}

impl<Args, Output> Execution<Args, Output, SolEncoding>
where
    // Args: scale::Encode,
    Args: SolValue,
    Output: SolValue + From<<<Output as SolValue>::SolType as SolType>::RustType>,
{
    /// Construct a new contract execution with the given input data.
    pub fn new(input: ExecutionInput<Args, SolEncoding>) -> Self {
        Self {
            input,
            output: ReturnType::default(),
        }
    }

    /// Perform the execution of the contract with the given executor.
    pub fn exec<I, E>(
        self,
        executor: &I,
    ) -> Result<ink_primitives::MessageResult<Output>, I::Error>
    where
        E: Environment,
        I: Executor<E>,
    {
        executor.exec(&self.input)
    }
}

/// Implemented in different environments to perform contract execution.
pub trait Executor<E: Environment> {
    /// The type of the error that can be returned during execution.
    type Error;
    /// Perform the contract execution with the given input data, and return the result.
    fn exec<Args, Output, Strategy>(
        &self,
        input: &ExecutionInput<Args, Strategy>,
    ) -> Result<ink_primitives::MessageResult<Output>, Self::Error>
    where
        // Args: scale::Encode,
        Args: EncodeWith<Strategy>,
        Output: SolValue + From<<<Output as SolValue>::SolType as SolType>::RustType>;
}

/// The input data for a smart contract execution.
#[derive(Clone, Default, Debug)]
pub struct ExecutionInput<Args, Strategy> {
    /// The selector for the smart contract execution.
    selector: Selector,
    /// The arguments of the smart contract execution.
    args: Args,
    _marker: PhantomData<Strategy>,
}

use ink_primitives::reflect::{
    EncodeWith,
    ScaleEncoding,
    SolEncoding,
};

impl ExecutionInput<EmptyArgumentList, SolEncoding> {
    /// Creates a new execution input with the given selector.
    #[inline]
    pub fn new(selector: Selector) -> Self {
        Self {
            selector,
            args: ArgumentList::empty(),
            _marker: Default::default(),
        }
    }

    /// Pushes an argument to the execution input.
    #[inline]
    pub fn push_arg<T>(
        self,
        arg: T,
    ) -> ExecutionInput<ArgumentList<Argument<T>, EmptyArgumentList>, SolEncoding>
    where
        T: SolValue,
    {
        ExecutionInput {
            selector: self.selector,
            args: self.args.push_arg(arg),
            _marker: Default::default(),
        }
    }
}

impl<Head, Rest> ExecutionInput<ArgumentList<Argument<Head>, Rest>, SolEncoding> {
    /// Pushes an argument to the execution input.
    #[inline]
    pub fn push_arg<T>(
        self,
        arg: T,
    ) -> ExecutionInput<ArgsList<T, ArgsList<Head, Rest>>, SolEncoding>
    where
        T: SolValue,
    {
        ExecutionInput {
            selector: self.selector,
            args: self.args.push_arg(arg),
            _marker: Default::default(),
        }
    }
}

impl<Args, Strategy> ExecutionInput<Args, Strategy> {
    /// Modify the selector.
    ///
    /// Useful when using the [`ExecutionInput`] generated as part of the
    /// `ContractRef`, but using a custom selector.
    pub fn update_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }
}

/// An argument list.
///
/// This type is constructed mainly at compile type via type constructions
/// to avoid having to allocate heap memory while constructing the encoded
/// arguments. The potentially heap allocating encoding is done right at the end
/// where we can leverage the static environmental buffer instead of allocating
/// heap memory.
#[derive(Clone, Default, Debug)]
pub struct ArgumentList<Head, Rest> {
    /// The first argument of the argument list.
    head: Head,
    /// All the rest arguments.
    rest: Rest,
}

/// Minor simplification of an argument list with a head and rest.
pub type ArgsList<Head, Rest> = ArgumentList<Argument<Head>, Rest>;

/// A single argument and its reference to a known value.
#[derive(Clone, Debug)]
pub struct Argument<T> {
    /// The reference to the known value.
    ///
    /// Used for the encoding at the end of the construction.
    arg: T,
}

impl<T> Argument<T> {
    /// Creates a new argument.
    #[inline]
    fn new(arg: T) -> Self {
        Self { arg }
    }
}

/// The end of an argument list.
#[derive(Clone, Default, Debug)]
pub struct ArgumentListEnd;

/// An empty argument list.
pub type EmptyArgumentList = ArgumentList<ArgumentListEnd, ArgumentListEnd>;

impl EmptyArgumentList {
    /// Creates a new empty argument list.
    #[inline]
    pub fn empty() -> EmptyArgumentList {
        ArgumentList {
            head: ArgumentListEnd,
            rest: ArgumentListEnd,
        }
    }

    /// Pushes the first argument to the empty argument list.
    #[inline]
    pub fn push_arg<T>(self, arg: T) -> ArgumentList<Argument<T>, Self>
    where
        T: SolValue,
    {
        ArgumentList {
            head: Argument::new(arg),
            rest: self,
        }
    }
}

impl<Head, Rest> ArgumentList<Argument<Head>, Rest> {
    /// Pushes another argument to the argument list.
    #[inline]
    pub fn push_arg<T>(self, arg: T) -> ArgumentList<Argument<T>, Self>
    where
        T: SolValue,
    {
        ArgumentList {
            head: Argument::new(arg),
            rest: self,
        }
    }
}

impl<T> SolValue for Argument<T>
where
    T: SolValue,
    Argument<T>: SolTypeValue<<T as SolValue>::SolType>,
{
    type SolType = <T as SolValue>::SolType;

    fn abi_encode(&self) -> Vec<u8> {
        <T as SolValue>::abi_encode(&self.arg)
    }
}

impl SolTypeValue<()> for EmptyArgumentList {
    fn stv_to_tokens(&self) -> <() as SolType>::Token<'_> {
        ()
    }

    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {}

    fn stv_eip712_data_word(&self) -> Word {
        Word::from_slice(&[])
    }
}

impl SolValue for EmptyArgumentList {
    type SolType = ();

    fn abi_encode(&self) -> Vec<u8> {
        Vec::new()
    }
}

// impl<T> scale::Encode for Argument<T>
// where
//     T: scale::Encode,
// {
//     #[inline]
//     fn size_hint(&self) -> usize {
//         <T as scale::Encode>::size_hint(&self.arg)
//     }
//
//     #[inline]
//     fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
//         <T as scale::Encode>::encode_to(&self.arg, output)
//     }
// }
//
// impl scale::Encode for EmptyArgumentList {
//     #[inline]
//     fn size_hint(&self) -> usize {
//         0
//     }
//
//     #[inline]
//     fn encode_to<O: scale::Output + ?Sized>(&self, _output: &mut O) {}
// }

// impl<Head, Rest> scale::Encode for ArgumentList<Argument<Head>, Rest>
// where
//     Head: scale::Encode,
//     Rest: scale::Encode,
// {
//     #[inline]
//     fn size_hint(&self) -> usize {
//         scale::Encode::size_hint(&self.head)
//             .checked_add(scale::Encode::size_hint(&self.rest))
//             .unwrap()
//     }
//
//     #[inline]
//     fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
//         // We reverse the order of encoding because we build up the list of
//         // arguments in reverse order, too. This way we encode the arguments
//         // in the same order in which they have been pushed to the argument list
//         // while the argument list itself organizes them in reverse order.
//         scale::Encode::encode_to(&self.rest, output);
//         scale::Encode::encode_to(&self.head, output);
//     }
// }
//
// impl<Args> scale::Encode for ExecutionInput<Args>
// where
//     Args: scale::Encode,
// {
//     #[inline]
//     fn size_hint(&self) -> usize {
//         scale::Encode::size_hint(&self.selector)
//             .checked_add(scale::Encode::size_hint(&self.args))
//             .unwrap()
//     }
//
//     #[inline]
//     fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
//         scale::Encode::encode_to(&self.selector, output);
//         scale::Encode::encode_to(&self.args, output);
//     }
// }

impl<Head, Rest> SolTypeValue<(Rest::SolType, Head::SolType)>
    for ArgumentList<Argument<Head>, Rest>
where
    Head: SolValue,
    Rest: SolValue,
{
    fn stv_to_tokens(
        &self,
    ) -> (
        <Rest::SolType as SolType>::Token<'_>,
        <Head::SolType as SolType>::Token<'_>,
    ) {
        (self.rest.stv_to_tokens(), self.head.arg.stv_to_tokens())
    }

    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        self.rest.stv_abi_encode_packed_to(out);
        self.head.arg.stv_abi_encode_packed_to(out);
    }

    fn stv_eip712_data_word(&self) -> Word {
        todo!("Implement EIP-712 encoding for ArgumentList")
    }
}

impl<Head, Rest> SolValue for ArgumentList<Argument<Head>, Rest>
where
    Head: SolValue,
    Rest: SolValue,
{
    type SolType = (Rest::SolType, Head::SolType);

    fn abi_encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.extend(Rest::abi_encode(&self.rest));
        encoded.extend(Head::abi_encode(&self.head.arg));
        encoded
    }
}

use ink_prelude::vec::Vec;

impl<Args, S> ExecutionInput<Args, S>
where
    Args: EncodeWith<S>,
{
    /// TODO (@peterwht): docs
    pub fn call_data(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.extend(self.selector.to_bytes());
        self.args.encode_with(&mut encoded);
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_exec_input_works() {
        let selector = Selector::new([0x01, 0x02, 0x03, 0x04]);
        let exec_input = ExecutionInput::new(selector);
        let encoded = scale::Encode::encode(&exec_input);
        assert!(!encoded.is_empty());
        let decoded = <Selector as scale::Decode>::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded, selector);
    }

    #[test]
    fn empty_args_works() {
        let empty_list = ArgumentList::empty();
        let encoded = scale::Encode::encode(&empty_list);
        assert_eq!(encoded, <Vec<u8>>::new());
    }

    #[test]
    fn single_argument_works() {
        let empty_list = ArgumentList::empty().push_arg(&1i32);
        let encoded = scale::Encode::encode(&empty_list);
        assert!(!encoded.is_empty());
        let decoded = <i32 as scale::Decode>::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded, 1i32);
    }

    #[test]
    fn multiple_arguments_works() {
        let empty_list = ArgumentList::empty()
            .push_arg(&42i32)
            .push_arg(&true)
            .push_arg(&[0x66u8; 4]);
        let encoded = scale::Encode::encode(&empty_list);
        assert!(!encoded.is_empty());
        let decoded =
            <(i32, bool, [u8; 4]) as scale::Decode>::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded, (42i32, true, [0x66; 4]));
    }
}
