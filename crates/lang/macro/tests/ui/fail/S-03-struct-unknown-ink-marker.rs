use ink_lang as ink;

#[ink::contract]
mod unknown_ink_marker_on_struct {
    #[ink(storage)]
    pub struct UnknownInkMarkerOnStruct {}

    #[ink(unknown_or_unsupported)]
    pub struct HasUnknownMarker {}

    impl UnknownInkMarkerOnStruct {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
