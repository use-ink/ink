// Copyright (C) Use Ink (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


/// Evaluates to `None` if the given expression is a `Option::None`.
///
/// # Note
///
/// This given expression is not required to be of type `Option`.
#[macro_export]
#[doc(hidden)]
macro_rules! as_option {
    (None $(,)?) => {{ None }};
    (&None $(,)?) => {{ None }};
    (Option::<$t:ty>::None $(,)?) => {{
        None::<&$t>
    }};
    (&Option::<$t:ty>::None $(,)?) => {{
        None::<&$t>
    }};

    // Special case for Some literals
    (Some($val:expr) $(,)?) => {{
        Some(&$val)
    }};
    (&Some($val:expr) $(,)?) => {{
        Some(&$val)
    }};
    
    ( &$local:expr $(,)? ) => {{
        Some(&$local)
    }};

    ( $local:expr $(,)? ) => {{
        Some(&$local)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn as_option_works() {
        assert_eq!(Some(&true), as_option!(true));
        assert_eq!(Some(&42), as_option!(42));
        assert_eq!(Some(&"Hello, World!"), as_option!("Hello, World!"));
        
        assert_eq!(Some(&()), as_option!(Some(())));
        assert_eq!(Some(&5), as_option!(Some(5)));
        assert_eq!(Some(&true), as_option!(Some(true)));
        assert_eq!(Some(&true), as_option!(&Some(true)));
        
        assert_eq!(None, as_option!(Option::<u32>::None));
        assert_eq!(None, as_option!(Option::<bool>::None));
        assert_eq!(None, as_option!(&Option::<bool>::None));
    }

    #[test]
    fn struct_fields_and_metadata_work() {
        struct TestStruct {
            field_1: u32,
            field_2: u64,
        }
    
        let test = TestStruct {
            field_1: 1,
            field_2: 2,
        };
    
        assert_eq!(Some(&test.field_1), as_option!(test.field_1));
        assert_eq!(Some(&test.field_1), as_option!(&test.field_1));
        assert_eq!(Some(&test.field_2), as_option!(test.field_2));
        assert_eq!(Some(&test.field_2), as_option!(&test.field_2));
    
        // This simulates the event_metadata.rs case that was failing
        #[derive(Debug)]
        struct EventField {
            value: u64,
        }
    
        // Test with temporary struct and field access - critical for Rust 2024
        let field_ref = as_option!(EventField { value: 123 }.value);
        assert_eq!(Some(&123), field_ref);
    }
    
    #[test]
    fn event_stable_field_pattern_works() {
        // This test simulates the exact pattern used in the event macro
        // where a field is bound to a variable and then wrapped in as_option
    
        struct EventStruct {
            field_1: u32,
            field_2: u64,
        }
    
        let event = EventStruct {
            field_1: 42,
            field_2: 100,
        };
    
        // This is how fields are processed in the event macro:
        let stable_field = event.field_1;
        assert_eq!(Some(&42), as_option!(stable_field));
    
        // Test with normal field access
        assert_eq!(Some(&100), as_option!(event.field_2));
    
        // Test with temporary values
        let get_value = || 123;
        let stable_field_2 = get_value();
        assert_eq!(Some(&123), as_option!(stable_field_2));
    }
}
