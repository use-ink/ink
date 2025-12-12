use super::*;

#[test]
fn it_works() {
    let mut incrementer = Incrementer::new();
    // Can call using universal call syntax using the trait.
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 0);
    <Incrementer as Increment>::inc(&mut incrementer);
    // Normal call syntax possible to as long as the trait is in scope.
    assert_eq!(incrementer.get(), 1);
}
