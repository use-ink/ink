// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use crate::call::Selector;

/// The input data for a smart contract execution.
#[derive(Debug)]
pub struct ExecutionInput<Args> {
    /// The selector for the smart contract execution.
    selector: Selector,
    /// The arguments of the smart contract execution.
    args: Args,
}

impl ExecutionInput<EmptyArgumentList> {
    /// Creates a new execution input with the given selector.
    #[inline]
    pub fn new(selector: Selector) -> Self {
        Self {
            selector,
            args: ArgumentList::empty(),
        }
    }

    /// Pushes an argument to the execution input.
    #[inline]
    pub fn push_arg<T>(
        self,
        arg: T,
    ) -> ExecutionInput<ArgumentList<Argument<T>, EmptyArgumentList>>
    where
        T: scale::Encode,
    {
        ExecutionInput {
            selector: self.selector,
            args: self.args.push_arg(arg),
        }
    }
}

impl<'a, Head, Rest> ExecutionInput<ArgumentList<Argument<Head>, Rest>> {
    /// Pushes an argument to the execution input.
    #[inline]
    pub fn push_arg<T>(self, arg: T) -> ExecutionInput<ArgsList<T, ArgsList<Head, Rest>>>
    where
        T: scale::Encode,
    {
        ExecutionInput {
            selector: self.selector,
            args: self.args.push_arg(arg),
        }
    }
}

/// An argument list.
///
/// This type is constructed mainly at compile type via type constructions
/// to avoid having to allocate heap memory while constructing the encoded
/// arguments. The potentially heap allocating encoding is done right at the end
/// where we can leverage the static environmental buffer instead of allocating
/// heap memory.
#[derive(Debug)]
pub struct ArgumentList<Head, Rest> {
    /// The first argument of the argument list.
    head: Head,
    /// All the rest arguments.
    rest: Rest,
}

/// Minor simplification of an argument list with a head and rest.
pub type ArgsList<Head, Rest> = ArgumentList<Argument<Head>, Rest>;

/// A single argument and its reference to a known value.
#[derive(Debug)]
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
#[derive(Debug)]
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
        T: scale::Encode,
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
        T: scale::Encode,
    {
        ArgumentList {
            head: Argument::new(arg),
            rest: self,
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

impl scale::Encode for EmptyArgumentList {
    #[inline]
    fn size_hint(&self) -> usize {
        0
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, _output: &mut O) {}
}

impl<'a, Head, Rest> scale::Encode for ArgumentList<Argument<Head>, Rest>
where
    Head: scale::Encode,
    Rest: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        scale::Encode::size_hint(&self.head) + scale::Encode::size_hint(&self.rest)
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

impl<Args> scale::Encode for ExecutionInput<Args>
where
    Args: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        scale::Encode::size_hint(&self.selector) + scale::Encode::size_hint(&self.args)
    }

    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
        scale::Encode::encode_to(&self.selector, output);
        scale::Encode::encode_to(&self.args, output);
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
