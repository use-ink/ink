use super::incrementer::Incrementer;

#[ink::test]
fn default_works() {
    // Given
    let contract = Incrementer::new_default();
    
    // Then
    assert_eq!(contract.get(), 0);
}

#[ink::test]
fn it_works() {
    // Given
    let mut contract = Incrementer::new(42);
    assert_eq!(contract.get(), 42);
    
    // When
    contract.inc(5);
    
    // Then
    assert_eq!(contract.get(), 47);
    
    // When
    contract.inc(-50);
    
    // Then
    assert_eq!(contract.get(), -3);
}