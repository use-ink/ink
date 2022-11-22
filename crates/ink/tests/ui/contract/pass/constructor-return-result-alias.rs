#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        Foo,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Result<Self> {
            Err(Error::Foo)
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

use ink::metadata::InkProject;

fn generate_metadata() -> InkProject {
    extern "Rust" {
        fn __ink_generate_metadata() -> InkProject;
    }

    unsafe { __ink_generate_metadata() }
}

fn main() {
    use contract::Contract;
    use std::any::TypeId;

    const ID: u32 = <Contract as ::ink::reflect::ContractDispatchableConstructors<
        { <Contract as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS },
    >>::IDS[0];

    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::IS_RESULT,
        true
    );
    assert_eq!(
        TypeId::of::<
            <Contract as ::ink::reflect::DispatchableConstructorInfo<{ ID }>>::Error,
        >(),
        TypeId::of::<contract::Error>(),
    );

    let metadata = generate_metadata();

    let constructor = metadata.spec().constructors().iter().next().unwrap();

    assert_eq!("constructor", constructor.label());
    let type_spec = constructor.return_type().opt_type().unwrap();
    assert_eq!(
        "ink_primitives::ConstructorResult",
        format!("{}", type_spec.display_name())
    );
    let ty = metadata.registry().resolve(type_spec.ty().id()).unwrap();

    assert_eq!("Result", format!("{}", ty.path()));
    match ty.type_def() {
        scale_info::TypeDef::Variant(variant) => {
            assert_eq!(2, variant.variants().len());

            // Outer Result
            let outer_ok_variant = &variant.variants()[0];
            let outer_ok_field = &outer_ok_variant.fields()[0];
            let outer_ok_ty = metadata
                .registry()
                .resolve(outer_ok_field.ty().id())
                .unwrap();
            assert_eq!("Ok", outer_ok_variant.name());

            // Inner Result
            let inner_ok_ty = match outer_ok_ty.type_def() {
                scale_info::TypeDef::Variant(variant) => {
                    assert_eq!(2, variant.variants().len());

                    let inner_ok_variant = &variant.variants()[0];
                    assert_eq!("Ok", inner_ok_variant.name());

                    let inner_ok_field = &inner_ok_variant.fields()[0];
                    metadata
                        .registry()
                        .resolve(inner_ok_field.ty().id())
                        .unwrap()
                }
                td => panic!("Expected a Variant type def enum, got {:?}", td),
            };

            let unit_ty = scale_info::TypeDef::Tuple(
                scale_info::TypeDefTuple::new_portable(vec![]),
            );

            assert_eq!(
                &unit_ty,
                inner_ok_ty.type_def(),
                "Ok variant should be a unit `()` type"
            );

            let err_variant = &variant.variants()[1];
            let err_field = &err_variant.fields()[0];
            let err_ty_result = metadata.registry().resolve(err_field.ty().id());
            assert_eq!("Err", err_variant.name());
            assert!(
                err_ty_result.is_some(),
                "Error variant must be encoded with SCALE"
            );
        }
        td => panic!("Expected a Variant type def enum, got {:?}", td),
    }
}
