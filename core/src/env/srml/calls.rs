// Copyright 2019 Parity Technologies (UK) Ltd.
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

use crate::env::EnvTypes;
use parity_codec::{
    Decode,
    Encode,
    Input,
    Output,
};
use core::convert::TryInto;

#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum Address<T: EnvTypes> {
    Id(T::AccountId),
    Index(T::AccountIndex),
}

fn need_more_than<T: PartialOrd>(a: T, b: T) -> Option<T> {
    if a < b { Some(b) } else { None }
}

/// Decode impl copied from
/// https://github.com/paritytech/substrate/blob/ec62d24c602912f07bbc416711376d9b8e5782c5/srml/indices/src/address.rs#L61
impl<T> Decode for Address<T> where
    T: EnvTypes,
{
    fn decode<I: Input>(input: &mut I) -> Option<Self> {
        Some(match input.read_byte()? {
            x @ 0x00..=0xef => Address::Index(T::AccountIndex::from(x as u32)),
            0xfc => Address::Index(T::AccountIndex::from(
                need_more_than(0xef, u16::decode(input)?)? as u32
            )),
            0xfd => Address::Index(T::AccountIndex::from(
                need_more_than(0xffff, u32::decode(input)?)?
            )),
            0xfe => Address::Index(
                need_more_than(0xffffffffu32.into(), Decode::decode(input)?)?
            ),
            0xff => Address::Id(Decode::decode(input)?),
            _ => return None,
        })
    }
}

/// Encode impl copied from
/// https://github.com/paritytech/substrate/blob/ec62d24c602912f07bbc416711376d9b8e5782c5/srml/indices/src/address.rs#L83
impl<T> Encode for Address<T> where
    T: EnvTypes,
{
    fn encode_to<O: Output>(&self, dest: &mut O) {
        match *self {
            Address::Id(ref i) => {
                dest.push_byte(255);
                dest.push(i);
            }
            Address::Index(i) => {
                let maybe_u32: Result<u32, _> = i.try_into();
                if let Ok(x) = maybe_u32 {
                    if x > 0xffff {
                        dest.push_byte(253);
                        dest.push(&x);
                    }
                    else if x >= 0xf0 {
                        dest.push_byte(252);
                        dest.push(&(x as u16));
                    }
                    else {
                        dest.push_byte(x as u8);
                    }

                } else {
                    dest.push_byte(254);
                    dest.push(&i);
                }
            },
        }
    }
}

#[derive(Encode)]
#[cfg_attr(feature = "test-env", derive(Decode, Debug, Clone, PartialEq, Eq))]
pub enum Balances<T: EnvTypes> {
    #[allow(non_camel_case_types)]
    transfer(Address<T>, #[codec(compact)] T::Balance),
    #[allow(non_camel_case_types)]
    set_balance(
        Address<T>,
        #[codec(compact)] T::Balance,
        #[codec(compact)] T::Balance,
    ),
}
