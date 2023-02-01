mod foo {
    #[ink::trait_definition(namespace = "foo")]
    pub trait TraitDefinition {
        #[ink(message)]
        fn message(&self);
    }
}

mod bar {
    #[ink::trait_definition(namespace = "bar")]
    pub trait TraitDefinition {
        #[ink(message)]
        fn message(&self);
    }
}

use bar::TraitDefinition as TraitDefinition2;
use foo::TraitDefinition as TraitDefinition1;
use ink::{
    reflect::{
        TraitDefinitionRegistry,
        TraitMessageInfo,
    },
    selector_bytes,
    selector_id,
};
use ink_env::DefaultEnvironment;

fn main() {
    macro_rules! assert_selector_eq {
        ( $trait_ident:path, $message_id:literal, $expected_selector:expr $(,)? ) => {
            assert_eq!(
                <<TraitDefinitionRegistry<DefaultEnvironment> as $trait_ident>::__ink_TraitInfo
                    as TraitMessageInfo<{selector_id!($message_id)}>>::SELECTOR,
                    $expected_selector
            );
        }
    }

    assert_selector_eq!(
        TraitDefinition1,
        "message",
        selector_bytes!("foo::TraitDefinition::message"),
    );
    assert_selector_eq!(
        TraitDefinition2,
        "message",
        selector_bytes!("bar::TraitDefinition::message"),
    );
}
