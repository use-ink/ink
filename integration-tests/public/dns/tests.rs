use super::dns::*;
use ink::{H256, primitives::Address};

// Helper to get default test accounts
fn default_accounts() -> ink::env::test::DefaultAccounts {
    ink::env::test::default_accounts()
}

// Helper to set the caller for the next contract execution
fn set_next_caller(caller: Address) {
    ink::env::test::set_caller(caller);
}

#[ink::test]
fn register_works() {
    // Given
    let default_accounts = default_accounts();
    let name = H256::from([0x99; 32]);

    set_next_caller(default_accounts.alice);
    let mut contract = DomainNameService::new();

    // When / Then
    // Registering a new name should succeed
    assert_eq!(contract.register(name), Ok(()));
    
    // Trying to register the same name again should fail
    assert_eq!(contract.register(name), Err(Error::NameAlreadyExists));
}

#[ink::test]
fn set_address_works() {
    // Given
    let accounts = default_accounts();
    let name = H256::from([0x99; 32]);

    set_next_caller(accounts.alice);
    let mut contract = DomainNameService::new();
    assert_eq!(contract.register(name), Ok(()));

    // When / Then
    // Caller is not owner (Bob), `set_address` should fail.
    set_next_caller(accounts.bob);
    assert_eq!(
        contract.set_address(name, accounts.bob),
        Err(Error::CallerIsNotOwner)
    );

    // Caller is owner (Alice), `set_address` should be successful
    set_next_caller(accounts.alice);
    assert_eq!(contract.set_address(name, accounts.bob), Ok(()));
    assert_eq!(contract.get_address(name), accounts.bob);
}

#[ink::test]
fn transfer_works() {
    // Given
    let accounts = default_accounts();
    let name = H256::from([0x99; 32]);

    set_next_caller(accounts.alice);
    let mut contract = DomainNameService::new();
    assert_eq!(contract.register(name), Ok(()));

    // When
    // Test transfer of owner from Alice to Bob.
    assert_eq!(contract.transfer(name, accounts.bob), Ok(()));

    // Then
    // Owner is now Bob, so Alice calling `set_address` should fail.
    assert_eq!(
        contract.set_address(name, accounts.bob),
        Err(Error::CallerIsNotOwner)
    );

    // Switch caller to Bob
    set_next_caller(accounts.bob);
    
    // Now owner is Bob, `set_address` should be successful.
    assert_eq!(contract.set_address(name, accounts.bob), Ok(()));
    assert_eq!(contract.get_address(name), accounts.bob);
}