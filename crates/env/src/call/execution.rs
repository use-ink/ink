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

use core::marker::PhantomData;
use ink_prelude::vec::Vec;
use ink_primitives::{
    SolEncode,
    abi::{
        AbiEncodeWith,
        Ink,
        SizedOutput,
        Sol,
    },
    impl_all_tuples,
    sol::SolTypeParamsEncode,
};

use super::{
    selector::Selector,
    utils::{
        DecodeMessageResult,
        ReturnType,
    },
};
use crate::Environment;

/// The input data and the expected return type of a contract execution.
pub struct Execution<Args, Output, Abi> {
    /// The input data for initiating a contract execution.
    pub input: ExecutionInput<Args, Abi>,
    /// The type of the expected return value of the contract execution.
    pub output: ReturnType<Output>,
}

impl<Args, Output, Abi> Execution<Args, Output, Abi>
where
    Args: EncodeArgsWith<Abi>,
    Output: DecodeMessageResult<Abi>,
{
    /// Construct a new contract execution with the given input data.
    pub fn new(input: ExecutionInput<Args, Abi>) -> Self {
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
    fn exec<Args, Output, Abi>(
        &self,
        input: &ExecutionInput<Args, Abi>,
    ) -> Result<ink_primitives::MessageResult<Output>, Self::Error>
    where
        Args: EncodeArgsWith<Abi>,
        Output: DecodeMessageResult<Abi>;
}

/// The input data for a smart contract execution.
#[derive(Clone, Default, Debug)]
pub struct ExecutionInput<Args, Abi> {
    /// The selector (if any) for the smart contract execution.
    selector: Option<Selector>,
    /// The arguments of the smart contract execution.
    args: Args,
    _marker: PhantomData<Abi>,
}

impl<Abi> ExecutionInput<EmptyArgumentList<Abi>, Abi> {
    /// Creates a new execution input with the given selector.
    #[inline]
    pub fn new(selector: Selector) -> Self {
        Self {
            selector: Some(selector),
            args: ArgumentList::empty(),
            _marker: Default::default(),
        }
    }
}

impl ExecutionInput<EmptyArgumentList<Sol>, Sol> {
    /// Creates a new execution input with no selector.
    ///
    /// # Note
    ///
    /// Should only be used for Solidity ABI encoded constructors/instantiation.
    #[inline]
    pub fn no_selector() -> Self {
        Self {
            selector: None,
            args: ArgumentList::empty(),
            _marker: Default::default(),
        }
    }
}

impl<Abi> ExecutionInput<EmptyArgumentList<Abi>, Abi> {
    /// Pushes an argument to the execution input.
    #[inline]
    pub fn push_arg<T>(
        self,
        arg: T,
    ) -> ExecutionInput<ArgumentList<Argument<T>, EmptyArgumentList<Abi>, Abi>, Abi>
    where
        T: AbiEncodeWith<Abi>,
    {
        ExecutionInput {
            selector: self.selector,
            args: self.args.push_arg(arg),
            _marker: Default::default(),
        }
    }
}

impl<Head, Rest, Abi> ExecutionInput<ArgumentList<Argument<Head>, Rest, Abi>, Abi> {
    /// Pushes an argument to the execution input.
    #[allow(clippy::type_complexity)]
    #[inline]
    pub fn push_arg<T>(
        self,
        arg: T,
    ) -> ExecutionInput<ArgsList<T, ArgsList<Head, Rest, Abi>, Abi>, Abi>
    where
        T: AbiEncodeWith<Abi>,
    {
        ExecutionInput {
            selector: self.selector,
            args: self.args.push_arg(arg),
            _marker: Default::default(),
        }
    }
}

impl<Args, Abi> ExecutionInput<Args, Abi> {
    /// Modify the selector.
    ///
    /// Useful when using the [`ExecutionInput`] generated as part of the
    /// `ContractRef`, but using a custom selector.
    pub fn update_selector(&mut self, selector: Selector) {
        self.selector = Some(selector);
    }
}

impl<Args, Abi> ExecutionInput<Args, Abi>
where
    Args: EncodeArgsWith<Abi>,
{
    /// Encodes the execution input into a dynamic vector by combining the selector and
    /// encoded arguments.
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        if let Some(selector) = &self.selector {
            encoded.extend(selector.to_bytes());
        }
        self.args.encode_to_vec(&mut encoded);
        encoded
    }

    /// Encodes the execution input into a static buffer by combining the selector and
    /// encoded arguments.
    pub fn encode_to_slice(&self, buffer: &mut [u8]) -> usize {
        let selector_len = if let Some(selector) = &self.selector {
            let selector_bytes = selector.to_bytes();
            let selector_len = selector_bytes.len();
            buffer[..selector_len].copy_from_slice(&selector_bytes);
            selector_len
        } else {
            0
        };
        let args_len = self.args.encode_to(&mut buffer[selector_len..]);
        selector_len + args_len
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
pub struct ArgumentList<Head, Rest, Abi> {
    /// The first argument of the argument list.
    head: Head,
    /// All the rest arguments.
    rest: Rest,
    _marker: PhantomData<Abi>,
}

/// Minor simplification of an argument list with a head and rest.
pub type ArgsList<Head, Rest, Abi> = ArgumentList<Argument<Head>, Rest, Abi>;

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
pub type EmptyArgumentList<Abi> = ArgumentList<ArgumentListEnd, ArgumentListEnd, Abi>;

impl<Abi> EmptyArgumentList<Abi> {
    /// Creates a new empty argument list.
    #[inline]
    pub fn empty() -> EmptyArgumentList<Abi> {
        ArgumentList {
            head: ArgumentListEnd,
            rest: ArgumentListEnd,
            _marker: Default::default(),
        }
    }

    /// Pushes the first argument to the empty argument list.
    #[inline]
    pub fn push_arg<T>(self, arg: T) -> ArgumentList<Argument<T>, Self, Abi>
    where
        T: AbiEncodeWith<Abi>,
    {
        ArgumentList {
            head: Argument::new(arg),
            rest: self,
            _marker: Default::default(),
        }
    }
}

impl<Head, Rest, Abi> ArgumentList<Argument<Head>, Rest, Abi> {
    /// Pushes another argument to the argument list.
    #[inline]
    pub fn push_arg<T>(self, arg: T) -> ArgumentList<Argument<T>, Self, Abi>
    where
        T: AbiEncodeWith<Abi>,
    {
        ArgumentList {
            head: Argument::new(arg),
            rest: self,
            _marker: Default::default(),
        }
    }
}

impl<T> scale::Encode for Argument<T>
where
    T: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        <T as scale::Encode>::size_hint(&self.arg)
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
        <T as scale::Encode>::encode_to(&self.arg, output)
    }
}

impl scale::Encode for EmptyArgumentList<Ink> {
    #[inline]
    fn size_hint(&self) -> usize {
        0
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, _output: &mut O) {}
}

impl<Head, Rest> scale::Encode for ArgumentList<Argument<Head>, Rest, Ink>
where
    Head: scale::Encode,
    Rest: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        scale::Encode::size_hint(&self.head)
            .checked_add(scale::Encode::size_hint(&self.rest))
            .expect("unable to checked_add")
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
        // We reverse the order of encoding because we build up the list of
        // arguments in reverse order, too. This way we encode the arguments
        // in the same order in which they have been pushed to the argument list
        // while the argument list itself organizes them in reverse order.
        scale::Encode::encode_to(&self.rest, output);
        scale::Encode::encode_to(&self.head, output);
    }
}

impl<Args> scale::Encode for ExecutionInput<Args, Ink>
where
    Args: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        let selector_size = match &self.selector {
            None => 0,
            Some(_) => scale::Encode::size_hint(&self.selector),
        };
        selector_size
            .checked_add(scale::Encode::size_hint(&self.args))
            .expect("unable to checked_add")
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
        if let Some(selector) = &self.selector {
            scale::Encode::encode_to(selector, output);
        }
        scale::Encode::encode_to(&self.args, output);
    }
}

impl<'a, T> SolEncode<'a> for Argument<T>
where
    T: SolEncode<'a>,
{
    type SolType = <T as SolEncode<'a>>::SolType;

    fn to_sol_type(&'a self) -> Self::SolType {
        self.arg.to_sol_type()
    }
}

/// Trait for encoding an arguments list as per the specified ABI.
pub trait EncodeArgsWith<Abi> {
    /// Encodes the data into a new vector.
    fn encode(&self) -> Vec<u8>;

    /// Encodes the data into a fixed-size buffer, returning the number of bytes written.
    fn encode_to(&self, buffer: &mut [u8]) -> usize;

    /// Encodes the data into a dynamically resizing vector.
    fn encode_to_vec(&self, buffer: &mut Vec<u8>);
}

impl<T: scale::Encode> EncodeArgsWith<Ink> for T {
    fn encode(&self) -> Vec<u8> {
        scale::Encode::encode(self)
    }

    fn encode_to(&self, buffer: &mut [u8]) -> usize {
        let mut sized_output = SizedOutput::from(buffer);
        scale::Encode::encode_to(self, &mut sized_output);
        sized_output.len()
    }

    fn encode_to_vec(&self, buffer: &mut Vec<u8>) {
        scale::Encode::encode_to(self, buffer);
    }
}

impl EncodeArgsWith<Sol> for EmptyArgumentList<Sol> {
    fn encode(&self) -> Vec<u8> {
        Vec::new()
    }

    fn encode_to(&self, _buffer: &mut [u8]) -> usize {
        0
    }

    fn encode_to_vec(&self, _buffer: &mut Vec<u8>) {}
}

impl<Head, Rest> EncodeArgsWith<Sol> for ArgumentList<Argument<Head>, Rest, Sol>
where
    for<'a> Self: ArgsListNestedTuple<'a>,
    for<'a> <Self as ArgsListNestedTuple<'a>>::OutputType: ArgsListFlatTuple,
    for<'a> <<Self as ArgsListNestedTuple<'a>>::OutputType as ArgsListFlatTuple>::OutputType:
        SolTypeParamsEncode,
{
    fn encode(&self) -> Vec<u8> {
        SolTypeParamsEncode::encode(&self.nested_tuple().flat_tuple())
    }

    fn encode_to(&self, buffer: &mut [u8]) -> usize {
        SolTypeParamsEncode::encode_to(&self.nested_tuple().flat_tuple(), buffer)
    }

    fn encode_to_vec(&self, buffer: &mut Vec<u8>) {
        buffer.extend(self.encode());
    }
}

/// Converts `ArgumentList` into a nested tuple representation.
trait ArgsListNestedTuple<'a> {
    type OutputType;

    fn nested_tuple(&'a self) -> Self::OutputType;
}

impl ArgsListNestedTuple<'_> for EmptyArgumentList<Sol> {
    type OutputType = ();

    fn nested_tuple(&self) {}
}

impl<'a, Head, Rest> ArgsListNestedTuple<'a> for ArgumentList<Argument<Head>, Rest, Sol>
where
    Rest: ArgsListNestedTuple<'a>,
    Head: SolEncode<'a>,
{
    type OutputType = (Rest::OutputType, <Head as SolEncode<'a>>::SolType);

    fn nested_tuple(&'a self) -> Self::OutputType {
        (self.rest.nested_tuple(), self.head.arg.to_sol_type())
    }
}

/// Converts an `ArgumentList` nested tuple into a flat tuple.
trait ArgsListFlatTuple {
    type OutputType;

    fn flat_tuple(self) -> Self::OutputType;
}

impl ArgsListFlatTuple for () {
    type OutputType = ();

    fn flat_tuple(self) {}
}

// Converts from `A, B, C` to `((((), A), B), C)`
macro_rules! args_list_nested_tuple {
    // Initialize.
    ($($ty:ident),* $(,)?) => {
        args_list_nested_tuple!(($($ty),*) @out: ())
    };
    // Process.
    (($Head:ident $(, $Rest:ident)*) @out: $Out:tt) => {
        args_list_nested_tuple!(($($Rest),*) @out: ($Out, $Head))
    };
    // Finalize.
    (() @out: $Out:tt) => { $Out };
}

macro_rules! impl_flat_tuple {
    ($( $ty: ident ),*) => {
        impl<$( $ty ),*> ArgsListFlatTuple for args_list_nested_tuple!($( $ty ),*) {
            type OutputType = ( $( $ty, )* );

            fn flat_tuple(self) -> Self::OutputType {
                #[allow(bad_style)]
                let args_list_nested_tuple!($( $ty ),*) = self;
                ( $( $ty, )* )
            }
        }
    };
}

impl_all_tuples!(@nonempty impl_flat_tuple);

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

    #[test]
    fn sol_empty_args_works() {
        let empty_list = ArgumentList::empty();
        let encoded = EncodeArgsWith::<Sol>::encode(&empty_list);
        assert_eq!(encoded, Vec::<u8>::new());
    }

    #[test]
    fn sol_single_argument_works() {
        let args_list = ArgumentList::empty().push_arg(&1i32);
        let encoded = EncodeArgsWith::<Sol>::encode(&args_list);
        assert!(!encoded.is_empty());
        let (decoded,) =
            ink_primitives::sol::decode_sequence::<(i32,)>(&encoded).unwrap();
        assert_eq!(decoded, 1i32);
    }

    #[test]
    fn sol_encoding_arguments_works() {
        let args_list = EmptyArgumentList::<Sol>::empty()
            .push_arg(100u8)
            .push_arg(vec![1, 2, 3, 4])
            .push_arg(String::from("Hello, world!"))
            .push_arg(true);
        let encoded_args_list = EncodeArgsWith::<Sol>::encode(&args_list);

        let args_tuple = (100u8, vec![1, 2, 3, 4], String::from("Hello, world!"), true);
        let encoded_args_tuple = ink_primitives::sol::encode_sequence(&args_tuple);

        assert_eq!(encoded_args_list, encoded_args_tuple);
    }
}
