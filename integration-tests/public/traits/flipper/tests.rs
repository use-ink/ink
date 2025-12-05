use super::Flip;
use super::flipper::Flipper;

#[::ink::test]
fn default_works() {
    // Given
    let flipper = Flipper::new();

    // Then
    assert!(flipper.get());
}

#[::ink::test]
fn it_works() {
    // Given
    let mut flipper = Flipper::new();

    // Then
    // Can call using universal call syntax using the trait.
    assert!(<Flipper as Flip>::get(&flipper));

    // When
    <Flipper as Flip>::flip(&mut flipper);

    // Then
    // Normal call syntax possible to as long as the trait is in scope.
    assert!(!flipper.get());
}