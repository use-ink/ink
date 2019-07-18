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
};

#[derive(Encode, Decode)]
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum Balances<T: EnvTypes> {
    #[allow(non_camel_case_types)]
    transfer(T::AccountIndex, #[codec(compact)] T::Balance),
    #[allow(non_camel_case_types)]
    set_balance(
        T::AccountIndex,
        #[codec(compact)] T::Balance,
        #[codec(compact)] T::Balance,
    ),
}
