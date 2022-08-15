mod arrays {
    use crate::item_works_for_primitive;

    type Array = [i32; 4];
    item_works_for_primitive!(Array);

    type ArrayTuples = [(i32, i32); 2];
    item_works_for_primitive!(ArrayTuples);
}

mod prims {
    use crate::item_works_for_primitive;
    use ink_env::AccountId;

    item_works_for_primitive!(bool);
    item_works_for_primitive!(String);
    item_works_for_primitive!(AccountId);
    item_works_for_primitive!(i8);
    item_works_for_primitive!(i16);
    item_works_for_primitive!(i32);
    item_works_for_primitive!(i64);
    item_works_for_primitive!(i128);
    item_works_for_primitive!(u8);
    item_works_for_primitive!(u16);
    item_works_for_primitive!(u32);
    item_works_for_primitive!(u64);
    item_works_for_primitive!(u128);

    type OptionU8 = Option<u8>;
    item_works_for_primitive!(OptionU8);

    type ResultU8 = Result<u8, bool>;
    item_works_for_primitive!(ResultU8);

    type BoxU8 = Box<u8>;
    item_works_for_primitive!(BoxU8);

    type BoxOptionU8 = Box<Option<u8>>;
    item_works_for_primitive!(BoxOptionU8);
}

mod tuples {
    use crate::item_works_for_primitive;

    type TupleSix = (i32, u32, String, u8, bool, Box<Option<i32>>);
    item_works_for_primitive!(TupleSix);
}
