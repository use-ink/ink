use ink_lang as ink;

#[ink::trait_definition]
pub trait PayableDefinition {
    #[ink(message, payable)]
    fn payable(&self);

    #[ink(message, payable)]
    fn payable_mut(&mut self);

    #[ink(message)]
    fn unpayable(&self);

    #[ink(message)]
    fn unpayable_mut(&mut self);
}

use ink_lang::selector_id;

const PAYABLE_ID: u32 = selector_id!("payable");
const PAYABLE_MUT_ID: u32 = selector_id!("payable_mut");
const UNPAYABLE_ID: u32 = selector_id!("unpayable");
const UNPAYABLE_MUT_ID: u32 = selector_id!("unpayable_mut");

fn main() {
    use ink_env::DefaultEnvironment;
    use ink_lang::reflect::{
        TraitDefinitionRegistry,
        TraitMessageInfo,
    };
    assert!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as TraitMessageInfo<PAYABLE_ID>>::PAYABLE,
    );
    assert!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as TraitMessageInfo<PAYABLE_MUT_ID>>::PAYABLE,
    );
    assert_eq!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as TraitMessageInfo<UNPAYABLE_ID>>::PAYABLE,
        false
    );
    assert_eq!(
        <<TraitDefinitionRegistry<DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as TraitMessageInfo<UNPAYABLE_MUT_ID>>::PAYABLE,
        false
    );
}
