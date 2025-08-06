#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::trait_definition]
pub trait Flip {
    /// Flips the current value of the Flipper's boolean.
    #[ink(message)]
    fn flip(&mut self);

    /// Returns the current value of the Flipper's boolean.
    #[ink(message)]
    fn get(&self) -> bool;

    #[cfg(feature = "foo")]
    #[ink(message)]
    fn push_foo(&mut self, value: bool);
}

#[ink::contract]
pub mod conditional_compilation {
    use super::Flip;

    /// Feature gated event
    #[cfg(feature = "foo")]
    #[ink(event)]
    pub struct Changes {
        // attributing event field with `cfg` is not allowed
        new_value: bool,
        #[ink(topic)]
        by: Address,
    }

    /// Feature gated event
    #[cfg(feature = "bar")]
    #[ink(event)]
    pub struct ChangesDated {
        // attributing event field with `cfg` is not allowed
        new_value: bool,
        #[ink(topic)]
        by: Address,
        when: BlockNumber,
    }

    #[ink(storage)]
    pub struct ConditionalCompilation {
        value: bool,
    }

    impl ConditionalCompilation {
        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                value: Default::default(),
            }
        }

        /// Constructor that is included when `foo` is enabled
        #[cfg(feature = "foo")]
        #[ink(constructor)]
        pub fn new_foo(value: bool) -> Self {
            Self { value }
        }

        /// Constructor that is included when `bar` is enabled
        #[cfg(feature = "bar")]
        #[ink(constructor)]
        pub fn new_bar(value: bool) -> Self {
            Self { value }
        }

        /// Constructor that is included with either `foo` or `bar` features enabled
        #[cfg(feature = "foo")]
        #[cfg(feature = "bar")]
        #[ink(constructor)]
        pub fn new_foo_bar(value: bool) -> Self {
            Self { value }
        }

        #[cfg(feature = "foo")]
        #[ink(message)]
        pub fn inherent_flip_foo(&mut self) {
            self.value = !self.value;
            let caller = Self::env().caller();
            Self::env().emit_event(Changes {
                new_value: self.value,
                by: caller,
            });
        }

        #[cfg(feature = "bar")]
        #[ink(message)]
        pub fn inherent_flip_bar(&mut self) {
            let caller = Self::env().caller();
            let block_number = Self::env().block_number();
            self.value = !self.value;
            Self::env().emit_event(ChangesDated {
                new_value: self.value,
                by: caller,
                when: block_number,
            });
        }
    }

    impl Flip for ConditionalCompilation {
        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            self.value
        }

        /// Feature gated mutating message
        #[cfg(feature = "foo")]
        #[ink(message)]
        fn push_foo(&mut self, value: bool) {
            let caller = Self::env().caller();
            Self::env().emit_event(Changes {
                new_value: value,
                by: caller,
            });
            self.value = value;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let flipper = ConditionalCompilation::new();
            assert!(!flipper.get());
        }

        #[ink::test]
        fn it_works() {
            let mut flipper = ConditionalCompilation::new();
            // Can call using universal call syntax using the trait.
            assert!(!<ConditionalCompilation as Flip>::get(&flipper));
            <ConditionalCompilation as Flip>::flip(&mut flipper);
            // Normal call syntax possible to as long as the trait is in scope.
            assert!(flipper.get());
        }

        #[cfg(feature = "foo")]
        #[ink::test]
        fn foo_works() {
            let mut flipper = ConditionalCompilation::new_foo(false);

            flipper.inherent_flip_foo();
            assert!(flipper.get());

            <ConditionalCompilation as Flip>::push_foo(&mut flipper, false);
            assert!(!flipper.get())
        }
    }
}
