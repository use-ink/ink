use super::incrementer::Incrementer;
use traits::Increment;

#[test]
fn default_works() {
    // Given
    let incrementer = Incrementer::new(0);
    
    // Then
    assert_eq!(incrementer.get(), 0);
}

#[test]
fn it_works() {
    // Given
    let mut incrementer = Incrementer::new(0);
    
    // Then
    // Can call using universal call syntax using the trait.
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 0);
    
    // When
    <Incrementer as Increment>::inc(&mut incrementer);
    
    // Then
    // Normal call syntax possible to as long as the trait is in scope.
    assert_eq!(incrementer.get(), 1);
}