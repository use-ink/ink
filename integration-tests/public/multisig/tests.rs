use super::multisig::*;
use ink::env::{
    call::utils::ArgumentList,
    test,
};
use ink::{Address, U256};

const WALLET: [u8; 20] = [7; 20];

impl Transaction {
    fn change_requirement(requirement: u32) -> Self {
        use ink::scale::Encode;
        let call_args = ArgumentList::empty().push_arg(&requirement);

        // Multisig::change_requirement()
        Self {
            callee: Address::from(WALLET),
            selector: ink::selector_bytes!(Abi::Ink, "change_requirement"),
            input: call_args.encode(),
            transferred_value: U256::zero(),
            ref_time_limit: 1000000,
            allow_reentry: false,
        }
    }
}

fn set_caller(sender: Address) {
    ink::env::test::set_caller(sender);
}

fn set_from_wallet() {
    let callee = Address::from(WALLET);
    set_caller(callee);
}

fn set_from_owner() {
    let accounts = default_accounts();
    set_caller(accounts.alice);
}

fn set_from_no_owner() {
    let accounts = default_accounts();
    set_caller(accounts.django);
}

fn default_accounts() -> test::DefaultAccounts {
    ink::env::test::default_accounts()
}

fn build_contract() -> Multisig {
    // Set the contract's address as `WALLET`.
    let callee: Address = Address::from(WALLET);
    ink::env::test::set_callee(callee);

    let accounts = default_accounts();
    let owners = vec![accounts.alice, accounts.bob, accounts.eve];
    Multisig::new(2, owners)
}

fn submit_transaction() -> Multisig {
    let mut contract = build_contract();
    let accounts = default_accounts();
    set_from_owner();
    contract.submit_transaction(Transaction::change_requirement(1));
    assert_eq!(contract.transaction_list.transactions.len(), 1);
    assert_eq!(test::recorded_events().len(), 2);
    let transaction = contract.transactions.get(0).unwrap();
    assert_eq!(transaction, Transaction::change_requirement(1));
    contract.confirmations.get((0, accounts.alice)).unwrap();
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
    contract
}

#[ink::test]
fn construction_works() {
    let accounts = default_accounts();
    let owners = [accounts.alice, accounts.bob, accounts.eve];
    let contract = build_contract();

    assert_eq!(contract.owners.len(), 3);
    assert_eq!(contract.requirement, 2);
    use ink::prelude::collections::HashSet;
    assert_eq!(
        HashSet::<&Address>::from_iter(contract.owners.iter()),
        HashSet::from_iter(owners.iter()),
    );
    assert!(contract.is_owner.contains(accounts.alice));
    assert!(contract.is_owner.contains(accounts.bob));
    assert!(contract.is_owner.contains(accounts.eve));
    assert!(!contract.is_owner.contains(accounts.charlie));
    assert!(!contract.is_owner.contains(accounts.django));
    assert!(!contract.is_owner.contains(accounts.frank));
    assert_eq!(contract.transaction_list.transactions.len(), 0);
}

#[ink::test]
#[should_panic]
fn empty_owner_construction_fails() {
    Multisig::new(0, vec![]);
}

#[ink::test]
#[should_panic]
fn zero_requirement_construction_fails() {
    let accounts = default_accounts();
    Multisig::new(0, vec![accounts.alice, accounts.bob]);
}

#[ink::test]
#[should_panic]
fn too_large_requirement_construction_fails() {
    let accounts = default_accounts();
    Multisig::new(3, vec![accounts.alice, accounts.bob]);
}

#[ink::test]
fn add_owner_works() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    let owners = contract.owners.len();
    contract.add_owner(accounts.frank);
    assert_eq!(contract.owners.len(), owners + 1);
    assert!(contract.is_owner.contains(accounts.frank));
    assert_eq!(test::recorded_events().len(), 1);
}

#[ink::test]
#[should_panic]
fn add_existing_owner_fails() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    contract.add_owner(accounts.bob);
}

#[ink::test]
#[should_panic]
fn add_owner_permission_denied() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_owner();
    contract.add_owner(accounts.frank);
}

#[ink::test]
fn remove_owner_works() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    let owners = contract.owners.len();
    contract.remove_owner(accounts.alice);
    assert_eq!(contract.owners.len(), owners - 1);
    assert!(!contract.is_owner.contains(accounts.alice));
    assert_eq!(test::recorded_events().len(), 1);
}

#[ink::test]
#[should_panic]
fn remove_owner_nonexisting_fails() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    contract.remove_owner(accounts.django);
}

#[ink::test]
#[should_panic]
fn remove_owner_permission_denied() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_owner();
    contract.remove_owner(accounts.alice);
}

#[ink::test]
fn replace_owner_works() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    let owners = contract.owners.len();
    contract.replace_owner(accounts.alice, accounts.django);
    assert_eq!(contract.owners.len(), owners);
    assert!(!contract.is_owner.contains(accounts.alice));
    assert!(contract.is_owner.contains(accounts.django));
    assert_eq!(test::recorded_events().len(), 2);
}

#[ink::test]
#[should_panic]
fn replace_owner_existing_fails() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    contract.replace_owner(accounts.alice, accounts.bob);
}

#[ink::test]
#[should_panic]
fn replace_owner_nonexisting_fails() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_wallet();
    contract.replace_owner(accounts.django, accounts.frank);
}

#[ink::test]
#[should_panic]
fn replace_owner_permission_denied() {
    let accounts = default_accounts();
    let mut contract = build_contract();
    set_from_owner();
    contract.replace_owner(accounts.alice, accounts.django);
}

#[ink::test]
fn change_requirement_works() {
    let mut contract = build_contract();
    assert_eq!(contract.requirement, 2);
    set_from_wallet();
    contract.change_requirement(3);
    assert_eq!(contract.requirement, 3);
    assert_eq!(test::recorded_events().len(), 1);
}

#[ink::test]
#[should_panic]
fn change_requirement_too_high() {
    let mut contract = build_contract();
    set_from_wallet();
    contract.change_requirement(4);
}

#[ink::test]
#[should_panic]
fn change_requirement_zero_fails() {
    let mut contract = build_contract();
    set_from_wallet();
    contract.change_requirement(0);
}

#[ink::test]
fn submit_transaction_works() {
    submit_transaction();
}

#[ink::test]
#[should_panic]
fn submit_transaction_no_owner_fails() {
    let mut contract = build_contract();
    set_from_no_owner();
    contract.submit_transaction(Transaction::change_requirement(1));
}

#[ink::test]
#[should_panic]
fn submit_transaction_wallet_fails() {
    let mut contract = build_contract();
    set_from_wallet();
    contract.submit_transaction(Transaction::change_requirement(1));
}

#[ink::test]
fn cancel_transaction_works() {
    let mut contract = submit_transaction();
    set_from_wallet();
    contract.cancel_transaction(0);
    assert_eq!(contract.transaction_list.transactions.len(), 0);
    assert_eq!(test::recorded_events().len(), 3);
}

#[ink::test]
fn cancel_transaction_nonexisting() {
    let mut contract = submit_transaction();
    set_from_wallet();
    contract.cancel_transaction(1);
    assert_eq!(contract.transaction_list.transactions.len(), 1);
    assert_eq!(test::recorded_events().len(), 2);
}

#[ink::test]
#[should_panic]
fn cancel_transaction_no_permission() {
    let mut contract = submit_transaction();
    contract.cancel_transaction(0);
}

#[ink::test]
fn confirm_transaction_works() {
    let mut contract = submit_transaction();
    let accounts = default_accounts();
    set_caller(accounts.bob);
    contract.confirm_transaction(0);
    assert_eq!(test::recorded_events().len(), 3);
    contract.confirmations.get((0, accounts.bob)).unwrap();
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 2);
}

#[ink::test]
fn revoke_confirmations() {
    // given
    let mut contract = submit_transaction();
    let accounts = default_accounts();
    // Confirm by Bob
    set_caller(accounts.bob);
    contract.confirm_transaction(0);
    // Confirm by Eve
    set_caller(accounts.eve);
    contract.confirm_transaction(0);
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 3);
    // Revoke from Eve
    contract.revoke_confirmation(0);
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 2);
    // Revoke from Bob
    set_caller(accounts.bob);
    contract.revoke_confirmation(0);
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
}

#[ink::test]
fn confirm_transaction_already_confirmed() {
    let mut contract = submit_transaction();
    let accounts = default_accounts();
    set_caller(accounts.alice);
    contract.confirm_transaction(0);
    assert_eq!(test::recorded_events().len(), 2);
    contract.confirmations.get((0, accounts.alice)).unwrap();
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
}

#[ink::test]
#[should_panic]
fn confirm_transaction_no_owner_fail() {
    let mut contract = submit_transaction();
    set_from_no_owner();
    contract.confirm_transaction(0);
}

#[ink::test]
fn revoke_transaction_works() {
    let mut contract = submit_transaction();
    let accounts = default_accounts();
    set_caller(accounts.alice);
    contract.revoke_confirmation(0);
    assert_eq!(test::recorded_events().len(), 3);
    assert!(!contract.confirmations.contains((0, accounts.alice)));
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 0);
}

#[ink::test]
fn revoke_transaction_no_confirmer() {
    let mut contract = submit_transaction();
    let accounts = default_accounts();
    set_caller(accounts.bob);
    contract.revoke_confirmation(0);
    assert_eq!(test::recorded_events().len(), 2);
    assert!(contract.confirmations.contains((0, accounts.alice)));
    assert_eq!(contract.confirmation_count.get(0).unwrap(), 1);
}

#[ink::test]
#[should_panic]
fn revoke_transaction_no_owner_fail() {
    let mut contract = submit_transaction();
    let accounts = default_accounts();
    set_caller(accounts.django);
    contract.revoke_confirmation(0);
}

#[ink::test]
fn execute_transaction_works() {
    // Execution of calls is currently unsupported in off-chain test.
    // Calling `execute_transaction` panics in any case.
}