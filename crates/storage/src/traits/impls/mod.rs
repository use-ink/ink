// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

// Collection works only with atomic structures
macro_rules! impl_always_storage_type {
    ( $name:ident < $( $frag:ident $( : $bound:tt $(+ $others:tt )* )? ),* > ) => {
        impl<
            Salt: $crate::traits::StorageKeyHolder,
            $($frag),+> $crate::traits::StorageType<Salt> for $name < $($frag),+ >
        where
            $(
                $frag: $crate::traits::AtomicGuard< { true } > $( + $bound $(+ $others )* )?,
            )+
        {
            type Type = $name < $($frag),+ >;
            type PreferredKey = $crate::traits::AutoKey;
        }
        impl<$($frag),+> $crate::traits::AtomicGuard< { true } >
            for $name < $($frag),+ >
            where
                $(
                    $frag: $crate::traits::AtomicGuard< { true } >,
                )+
            {}
    };
    ( $name:ty ) => {
        impl<
            Salt: $crate::traits::StorageKeyHolder,
            > $crate::traits::StorageType<Salt> for $name
        {
            type Type = $name;
            type PreferredKey = $crate::traits::AutoKey;
        }
        impl $crate::traits::AtomicGuard< { true } > for $name {}
    };
}

mod arrays;
mod collections;
mod prims;
mod tuples;

pub(crate) mod storage;
