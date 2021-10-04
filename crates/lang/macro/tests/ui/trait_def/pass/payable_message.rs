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

/// Computed using `local_id("payable")`.
const PAYABLE_ID: u32 = 0xFDDBE704;

/// Computed using `local_id("payable_mut")`.
const PAYABLE_MUT_ID: u32 = 0x1CE6275F;

/// Computed using `local_id("unpayable")`.
const UNPAYABLE_ID: u32 = 0x511647A5;

/// Computed using `local_id("unpayable_mut")`.
const UNPAYABLE_MUT_ID: u32 = 0x4A60F1E1;

fn main() {
    use ink_lang_ir as ir;
    /// Returns the local ID for the given name.
    #[allow(dead_code)]
    fn local_id(name: &str) -> u32 {
        ir::utils::local_message_id(&syn::Ident::new(
            name,
            proc_macro2::Span::call_site(),
        ))
    }
    // Uncomment these in order to print out the local IDs of
    // all the ink! trait messages for this test.
    //
    // println!("local_id(\"payable\")       = {:X}", local_id("payable"));
    // println!("local_id(\"payable_mut\")   = {:X}", local_id("payable_mut"));
    // println!("local_id(\"unpayable\")     = {:X}", local_id("unpayable"));
    // println!("local_id(\"unpayable_mut\") = {:X}", local_id("unpayable_mut"));
    assert!(
        <<::ink_lang::reflect::TraitDefinitionRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::reflect::TraitMessageInfo<PAYABLE_ID>>::PAYABLE,
    );
    assert!(
        <<::ink_lang::reflect::TraitDefinitionRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::reflect::TraitMessageInfo<PAYABLE_MUT_ID>>::PAYABLE,
    );
    assert_eq!(
        <<::ink_lang::reflect::TraitDefinitionRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::reflect::TraitMessageInfo<UNPAYABLE_ID>>::PAYABLE,
        false
    );
    assert_eq!(
        <<::ink_lang::reflect::TraitDefinitionRegistry<::ink_env::DefaultEnvironment>
            as PayableDefinition>::__ink_TraitInfo
            as ::ink_lang::reflect::TraitMessageInfo<UNPAYABLE_MUT_ID>>::PAYABLE,
        false
    );
}
