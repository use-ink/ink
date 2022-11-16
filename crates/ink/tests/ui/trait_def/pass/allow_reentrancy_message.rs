#[ink::trait_definition]
pub trait AllowReentrancyDefinition {
    #[ink(message, allow_reentrancy)]
    fn allow_reentrancy(&self);

    #[ink(message, allow_reentrancy)]
    fn allow_reentrancy_mut(&mut self);

    #[ink(message)]
    fn reentrancy_not_allowed(&self);

    #[ink(message)]
    fn reentrancy_not_allowed_mut(&mut self);
}

use ink::selector_id;

const ALLOW_REENTRANCY_ID: u32 = selector_id!("allow_reentrancy");
const ALLOW_REENTRANCY_MUT_ID: u32 = selector_id!("allow_reentrancy_mut");
const REENTRANCY_NOT_ALLOWED_ID: u32 = selector_id!("reentrancy_not_allowed");
const REENTRANCY_NOT_ALLOWED_MUT_ID: u32 = selector_id!("reentrancy_not_allowed_mut");

fn main() {
    use ink::reflect::{
        TraitDefinitionRegistry,
        TraitMessageInfo,
    };
    use ink_env::DefaultEnvironment;
    assert!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
            as AllowReentrancyDefinition>::__ink_TraitInfo
            as TraitMessageInfo<ALLOW_REENTRANCY_ID>>::ALLOW_REENTRANCY,
    );
    assert!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
            as AllowReentrancyDefinition>::__ink_TraitInfo
            as TraitMessageInfo<ALLOW_REENTRANCY_MUT_ID>>::ALLOW_REENTRANCY,
    );
    assert_eq!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
        as AllowReentrancyDefinition>::__ink_TraitInfo
        as TraitMessageInfo<REENTRANCY_NOT_ALLOWED_ID>>::ALLOW_REENTRANCY,
        false
    );
    assert_eq!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
        as AllowReentrancyDefinition>::__ink_TraitInfo
        as TraitMessageInfo<REENTRANCY_NOT_ALLOWED_MUT_ID>>::ALLOW_REENTRANCY,
        false
    );
}
