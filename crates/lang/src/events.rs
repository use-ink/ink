// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use scale;

/// Implemented by contracts in order to override `env().emit_event(..)`
/// syntax for emitting of ink! contract events.
///
/// # Dev Note
///
/// Normally we'd try to define traits like these in the compagnion
/// `ink_lang` crate, however, due to Rust's orphan rules we must
/// define this trait here.
pub trait EmitEvent<C>
where
    C: BaseEvent,
{
    /// Emits an event that can be trivially converted into the base event.
    fn emit_event<E>(self, event: E)
    where
        E: Into<<C as BaseEvent>::Type>;
}

/// Defines a base event type for the contract.
///
/// This is usually the event enum that comprises all defined event types.
pub trait BaseEvent {
    /// The generated base event enum.
    type Type;
}

pub struct PrefixedValue<'a, 'b, T> {
    pub prefix: &'a [u8],
    pub value: &'b T,
}

impl<X> scale::Encode for PrefixedValue<'_, '_, X>
where
    X: scale::Encode,
{
    #[inline(always)]
    fn size_hint(&self) -> usize {
        self.prefix.encode().len() + self.value.encode().len()
    }

    #[inline]
    fn encode_to<T: scale::Output>(&self, dest: &mut T) {
        dest.write(self.prefix.encode_to(dest));
        dest.write(self.value.encode_to(dest));
    }
}
