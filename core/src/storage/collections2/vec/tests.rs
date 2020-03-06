use super::Vec as StorageVec;

#[test]
fn new_vec_works() {
    let vec = <StorageVec<i32>>::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert!(vec.iter().next().is_none());
    let default = <StorageVec<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert!(default.iter().next().is_none());
}
