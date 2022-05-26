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

use crate::layout::{
    Layout,
    MetadataError,
    StructLayout,
};
use ink_prelude::collections::HashMap;
use ink_primitives::Key;
use scale_info::form::MetaForm;

/// It validates that the storage layout:
/// - Hasn't conflicting storage keys, otherwise returns an error with a description of the conflict
pub struct ValidateLayout {
    first_entry: HashMap<Key, String>,
    name_stack: Vec<String>,
}

impl ValidateLayout {
    /// Validates the storage layout
    pub fn validate(layout: &Layout<MetaForm>) -> Result<(), MetadataError> {
        let mut validator = Self {
            first_entry: Default::default(),
            name_stack: Default::default(),
        };
        validator.recursive_validate(layout)
    }

    fn recursive_validate(
        &mut self,
        layout: &Layout<MetaForm>,
    ) -> Result<(), MetadataError> {
        match layout {
            Layout::Root(root) => {
                self.check_key(root.root_key.key())?;
                self.recursive_validate(root.layout())
            }
            Layout::Hash(hash) => self.recursive_validate(hash.layout()),
            Layout::Array(array) => self.recursive_validate(array.layout()),
            Layout::Struct(st) => self.check_struct_layout(st),
            Layout::Enum(en) => {
                // After `Enum::` we will have the struct -> `Enum::Struct`
                self.name_stack.push(format!("{}::", en.name()));
                for variant in en.variants().values() {
                    self.check_struct_layout(variant)?;
                }
                self.name_stack.pop().unwrap();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn check_struct_layout(&mut self, st: &StructLayout) -> Result<(), MetadataError> {
        self.name_stack.push(st.name().to_string());
        for layout in st.fields() {
            let name = layout.name();
            // After `Struct` we always have fields -> `Struct.field`
            // After field we have `Struct` or `Enum` -> `Struct.field:Struct`
            self.name_stack.push(format!(".{}:", name));

            self.recursive_validate(layout.layout())?;

            self.name_stack.pop().unwrap();
        }
        self.name_stack.pop().unwrap();
        Ok(())
    }

    fn check_key(&mut self, key: &Key) -> Result<(), MetadataError> {
        let path = self.name_stack.join("");
        if let Some(prev_path) = self.first_entry.get(key) {
            Err(MetadataError::ConflictKey(prev_path.clone(), path))
        } else {
            self.first_entry.insert(*key, path);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::{
        CellLayout,
        EnumLayout,
        FieldLayout,
        Layout,
        MetadataError,
        RootLayout,
        StructLayout,
        ValidateLayout,
    };
    use ink_primitives::Key;
    use std::collections::BTreeSet;

    #[test]
    fn valid_layout_tree_only_roots() {
        // Root(0) -> Root(1) -> Root(2) -> u32
        let layout = RootLayout::new(
            0.into(),
            RootLayout::new(
                1.into(),
                RootLayout::new(2.into(), CellLayout::new::<u32>(2.into())),
            ),
        );

        assert!(ValidateLayout::validate(&Layout::Root(layout)).is_ok())
    }

    // If any of the root key are equal it should cause an error
    fn valid_big_layout_tree(
        key_for_root_0: Key,
        key_for_root_1: Key,
        key_for_root_2: Key,
        key_for_root_3: Key,
        key_for_root_4: Key,
    ) -> Result<(), MetadataError> {
        let root_0 = key_for_root_0.into();
        let root_1 = key_for_root_1.into();
        let root_2 = key_for_root_2.into();
        let root_3 = key_for_root_3.into();
        let root_4 = key_for_root_4.into();
        // Below the description of the layout tree. Inside `(...)` the expected storage key.
        //              Root(0)
        //                |
        //            Contract(0)
        //          /     |     \
        //  a:Root(1)  b:u32(0)  c:Struct0(0)
        //         |               /       \
        //  Vec<u8>(1)     d:u128(0)     f:Root(2)
        //                                   |
        //                                Enum(2)
        //                               /   |   \
        //                           First Second Third
        //                       0.Struct1 0.u8(2) 0.Root(3)
        //                             |            |
        //                           Root(4)      String
        //                             |
        //                      g:BTreeSet<u64>(4)
        let layout = RootLayout::new(
            root_0,
            StructLayout::new(
                "Contract",
                vec![
                    FieldLayout::new(
                        "a",
                        StructLayout::new(
                            "Struct0",
                            vec![
                                FieldLayout::new("d", CellLayout::new::<u128>(root_0)),
                                FieldLayout::new(
                                    "f",
                                    RootLayout::new(
                                        root_2,
                                        EnumLayout::new(
                                            "Enum",
                                            root_2,
                                            vec![
                                                (
                                                    0.into(),
                                                    StructLayout::new(
                                                        "First",
                                                        vec![FieldLayout::new(
                                                            "0",
                                                            StructLayout::new(
                                                                "Struct1",
                                                                vec![FieldLayout::new(
                                                                    "g",
                                                                    RootLayout::new(
                                                                        root_4,
                                                                        CellLayout::new::<
                                                                            BTreeSet<u64>,
                                                                        >(
                                                                            root_4
                                                                        ),
                                                                    ),
                                                                )],
                                                            ),
                                                        )],
                                                    ),
                                                ),
                                                (
                                                    1.into(),
                                                    StructLayout::new(
                                                        "Second",
                                                        vec![FieldLayout::new(
                                                            "0",
                                                            CellLayout::new::<u8>(root_2),
                                                        )],
                                                    ),
                                                ),
                                                (
                                                    2.into(),
                                                    StructLayout::new(
                                                        "Third",
                                                        vec![FieldLayout::new(
                                                            "0",
                                                            RootLayout::new(
                                                                root_3,
                                                                CellLayout::new::<String>(
                                                                    root_3,
                                                                ),
                                                            ),
                                                        )],
                                                    ),
                                                ),
                                            ],
                                        ),
                                    ),
                                ),
                            ],
                        ),
                    ),
                    FieldLayout::new("b", CellLayout::new::<u32>(root_0)),
                    FieldLayout::new(
                        "c",
                        RootLayout::new(root_1, CellLayout::new::<Vec<u8>>(root_1)),
                    ),
                ],
            ),
        );

        ValidateLayout::validate(&Layout::Root(layout))
    }

    #[test]
    fn tree_is_valid() {
        assert_eq!(Ok(()), valid_big_layout_tree(0, 1, 2, 3, 4));
        assert_eq!(Ok(()), valid_big_layout_tree(4, 3, 2, 1, 0));
    }

    #[test]
    fn conflict_0_and_1() {
        assert_eq!(
            Err(MetadataError::ConflictKey(
                "".to_string(),
                "Contract.c:".to_string()
            )),
            valid_big_layout_tree(0, 0, 2, 3, 4)
        )
    }

    #[test]
    fn conflict_0_and_2() {
        assert_eq!(
            Err(MetadataError::ConflictKey(
                "".to_string(),
                "Contract.a:Struct0.f:".to_string()
            )),
            valid_big_layout_tree(0, 1, 0, 3, 4)
        )
    }

    #[test]
    fn conflict_0_and_3() {
        assert_eq!(
            Err(MetadataError::ConflictKey(
                "".to_string(),
                "Contract.a:Struct0.f:Enum::Third.0:".to_string()
            )),
            valid_big_layout_tree(0, 1, 2, 0, 4)
        )
    }

    #[test]
    fn conflict_0_and_4() {
        assert_eq!(
            Err(MetadataError::ConflictKey(
                "".to_string(),
                "Contract.a:Struct0.f:Enum::First.0:Struct1.g:".to_string()
            )),
            valid_big_layout_tree(0, 1, 2, 3, 0)
        )
    }

    #[test]
    fn conflict_3_and_4() {
        assert_eq!(
            Err(MetadataError::ConflictKey(
                "Contract.a:Struct0.f:Enum::First.0:Struct1.g:".to_string(),
                "Contract.a:Struct0.f:Enum::Third.0:".to_string()
            )),
            valid_big_layout_tree(0, 1, 2, 3, 3)
        )
    }
}
