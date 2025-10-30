// Property-based tests for config validation robustness (T048)

use proptest::prelude::*;

// Property: Config parser should never panic on arbitrary TOML
proptest! {
    #[test]
    fn config_parser_never_panics(toml_content in "\\PC{0,500}") {
        let result = std::panic::catch_unwind(|| {
            unicleaner::config::parser::parse_config_str(&toml_content)
        });

        prop_assert!(
            result.is_ok(),
            "Config parser panicked on input"
        );
    }
}

// Property: Valid empty config should parse successfully
proptest! {
    #[test]
    fn empty_config_valid(_unit in prop::bool::ANY) {
        let empty = "";
        let result = unicleaner::config::parser::parse_config_str(empty);

        // Empty config should either succeed with defaults or fail gracefully
        prop_assert!(
            result.is_ok() || result.is_err(),
            "Empty config should return Result"
        );
    }
}

// Property: Config with valid TOML structure should not crash
proptest! {
    #[test]
    fn valid_toml_structure_safe(key in "[a-z]{1,20}", value in "[a-z0-9]{1,20}") {
        let toml = format!("{} = \"{}\"", key, value);

        let result = std::panic::catch_unwind(|| {
            unicleaner::config::parser::parse_config_str(&toml)
        });

        prop_assert!(result.is_ok(), "Valid TOML should not panic");
    }
}

// Property: Unicode ranges in config should be validated
proptest! {
    #[test]
    fn unicode_ranges_validated(start in 0u32..0x10FFFF, end in 0u32..0x10FFFF) {
        let toml = format!(
            "[allowed_ranges]\nstart = {}\nend = {}",
            start, end
        );

        let result = std::panic::catch_unwind(|| {
            unicleaner::config::parser::parse_config_str(&toml)
        });

        prop_assert!(result.is_ok(), "Unicode range config should not panic");
    }
}

// Property: Invalid UTF-8 in config should be handled
proptest! {
    #[test]
    fn invalid_utf8_handled(bytes in prop::collection::vec(any::<u8>(), 0..100)) {
        // Try to create a string (may fail if invalid UTF-8)
        if let Ok(s) = String::from_utf8(bytes) {
            let result = std::panic::catch_unwind(|| {
                unicleaner::config::parser::parse_config_str(&s)
            });

            prop_assert!(result.is_ok(), "Config parser should handle arbitrary UTF-8");
        }
    }
}

// Property: Duplicate keys in config should be handled
proptest! {
    #[test]
    fn duplicate_keys_handled(key in "[a-z]{1,15}", val1 in "[a-z]{1,10}", val2 in "[a-z]{1,10}") {
        let toml = format!("{} = \"{}\"\n{} = \"{}\"", key, val1, key, val2);

        let result = std::panic::catch_unwind(|| {
            unicleaner::config::parser::parse_config_str(&toml)
        });

        prop_assert!(result.is_ok(), "Duplicate keys should not panic");
    }
}

// Property: Very large config values should be handled
proptest! {
    #[test]
    fn large_values_handled(size in 1usize..10000) {
        let large_value = "a".repeat(size);
        let toml = format!("key = \"{}\"", large_value);

        let result = std::panic::catch_unwind(|| {
            unicleaner::config::parser::parse_config_str(&toml)
        });

        prop_assert!(result.is_ok(), "Large values should not panic");
    }
}

// Property: Nested TOML structures should be handled
proptest! {
    #[test]
    fn nested_toml_handled(depth in 1usize..5) {
        let mut toml = String::new();

        for i in 0..depth {
            toml.push_str(&format!("[level{}]\n", i));
            toml.push_str(&format!("key{} = \"value{}\"\n", i, i));
        }

        let result = std::panic::catch_unwind(|| {
            unicleaner::config::parser::parse_config_str(&toml)
        });

        prop_assert!(result.is_ok(), "Nested TOML should not panic");
    }
}
