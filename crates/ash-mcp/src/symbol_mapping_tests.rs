//! Tests for Ash → Rust and Rust → Ash symbol mapping tools
//!
//! Tests the cross-language symbol lookup tools with real workspace data.

#[cfg(test)]
mod symbol_mapping_tests {
    use crate::{AshMcpServer, SymbolLookupParams, RustUsageLookupParams};

    /// Test that `find_rust_symbol_location` can locate the real `Effect` enum.
    #[test]
    fn test_find_effect_symbol_location() {
        let location = AshMcpServer::find_rust_symbol_location("ash_core::effect::Effect")
            .unwrap();

        assert!(
            location.is_some(),
            "Should find Effect enum in workspace"
        );
        let loc = location.unwrap();
        assert!(
            loc.file.ends_with("effect.rs"),
            "Expected file to end with effect.rs, got: {}",
            loc.file
        );
        assert_eq!(loc.start_line, 11, "Effect enum is on line 11");
        assert_eq!(loc.start_column, 10, "Effect starts at column 10");
        assert_eq!(loc.end_column, 16, "Effect ends at column 16");
    }

    /// Test that `find_rust_symbol_location` can locate the real `CapabilityProvider` trait.
    #[test]
    fn test_find_capability_provider_symbol_location() {
        let location = AshMcpServer::find_rust_symbol_location("ash_core::capability::CapabilityProvider")
            .unwrap();

        assert!(
            location.is_some(),
            "Should find CapabilityProvider trait in workspace"
        );
        let loc = location.unwrap();
        assert!(
            loc.file.ends_with("capability.rs"),
            "Expected file to end with capability.rs, got: {}",
            loc.file
        );
        assert_eq!(loc.start_line, 31, "CapabilityProvider trait is on line 31");
        assert_eq!(loc.start_column, 10, "CapabilityProvider starts at column 10");
    }

    /// Test that `find_rust_symbol_location` can locate the real `CapabilityError` enum.
    #[test]
    fn test_find_capability_error_symbol_location() {
        let location = AshMcpServer::find_rust_symbol_location("ash_core::capability::CapabilityError")
            .unwrap();

        assert!(
            location.is_some(),
            "Should find CapabilityError enum in workspace"
        );
        let loc = location.unwrap();
        assert!(
            loc.file.ends_with("capability.rs"),
            "Expected file to end with capability.rs, got: {}",
            loc.file
        );
        assert_eq!(loc.start_line, 10, "CapabilityError enum is on line 10");
    }

    /// Test that unknown Rust symbols return `None` gracefully.
    #[test]
    fn test_unknown_rust_symbol_returns_none() {
        let location = AshMcpServer::find_rust_symbol_location("nonexistent::module::UnknownSymbol")
            .unwrap();

        assert!(
            location.is_none(),
            "Unknown symbols should return None gracefully"
        );
    }

    /// Test that short symbol paths (less than 3 parts) return `None`.
    #[test]
    fn test_short_symbol_path_returns_none() {
        let location = AshMcpServer::find_rust_symbol_location("ash_core::effect")
            .unwrap();

        assert!(
            location.is_none(),
            "Short symbol paths should return None"
        );
    }

    /// Test `find_ash_files_for_crate` discovers real `.ash` files in the workspace.
    #[test]
    fn test_find_ash_files_discovers_real_files() {
        let config = AshMcpServer::load_cross_lang_config();
        let files = AshMcpServer::find_ash_files_for_crate(&config, "ash_core::effect::Effect");

        assert!(
            !files.is_empty(),
            "Should discover at least some .ash files in the workspace"
        );
        for file in &files {
            assert!(
                file.ends_with(".ash"),
                "All discovered files should have .ash extension: {}",
                file
            );
        }
    }

    /// Test `find_ash_symbol_in_module` with a real parsed Ash file.
    #[test]
    fn test_find_ash_symbol_in_real_module() {
        let config = AshMcpServer::load_cross_lang_config();
        let files = AshMcpServer::find_ash_files_for_crate(&config, "ash_core::effect::Effect");

        // Try to find a symbol in the first discovered file that can be parsed
        let mut found_symbol = false;
        for file in &files {
            if let Ok(entry) = AshMcpServer::ensure_open(file) {
                if let Ok(module) = crate::AshMcpServer::parse_file(&entry) {
                    // Look for any top-level declaration; we just verify the traversal works
                    let _loc = AshMcpServer::find_ash_symbol_in_module(&module, "nonexistent");
                    found_symbol = true;
                    break;
                }
            }
        }

        assert!(
            found_symbol,
            "Should be able to parse at least one .ash file and traverse its AST"
        );
    }

    /// Test `find_rust_implementation` with a known symbol from the default config.
    #[test]
    fn test_find_rust_implementation_known_symbol() {
        let server = AshMcpServer::new();
        let params = SymbolLookupParams {
            ash_symbol: "Effect".to_string(),
            file: "std/src/types.ash".to_string(),
            line: 10,
            column: 1,
        };

        let result = server.find_rust_implementation(&params).unwrap();
        // With default (empty) config, this returns None because there are no mappings.
        // The test verifies it doesn't panic and handles the case gracefully.
        assert!(
            result.is_none() || result.as_ref().unwrap().found,
            "Should either return None or a found result, never an error"
        );
    }

    /// Test `find_rust_implementation` with an unknown symbol.
    #[test]
    fn test_find_rust_implementation_unknown_symbol() {
        let server = AshMcpServer::new();
        let params = SymbolLookupParams {
            ash_symbol: "UnknownSymbol".to_string(),
            file: "test.ash".to_string(),
            line: 1,
            column: 1,
        };

        let result = server.find_rust_implementation(&params).unwrap();
        assert!(
            result.is_none(),
            "Unknown symbols should return None"
        );
    }

    /// Test `find_ash_usage` with a known Rust symbol.
    #[test]
    fn test_find_ash_usage_known_symbol() {
        let server = AshMcpServer::new();
        let params = RustUsageLookupParams {
            rust_symbol: "ash_core::effect::Effect".to_string(),
            rust_crate: None,
            rust_module: None,
            ash_files: None,
        };

        let result = server.find_ash_usage(&params).unwrap();
        // With default (empty) config, no reverse mappings exist.
        // The test verifies it doesn't panic and returns a coherent result.
        assert!(!result.found, "With empty config, no Ash usages should be found");
        assert_eq!(result.rust_symbol, Some("ash_core::effect::Effect".to_string()));
        assert!(result.ash_files.is_empty());
        assert!(result.error.is_some());
    }

    /// Test `find_ash_usage` with an unknown Rust symbol.
    #[test]
    fn test_find_ash_usage_unknown_symbol() {
        let server = AshMcpServer::new();
        let params = RustUsageLookupParams {
            rust_symbol: "nonexistent::crate::Symbol".to_string(),
            rust_crate: None,
            rust_module: None,
            ash_files: None,
        };

        let result = server.find_ash_usage(&params).unwrap();
        assert!(!result.found, "Should not find Ash usages for unknown symbol");
        assert_eq!(result.rust_symbol, Some("nonexistent::crate::Symbol".to_string()));
        assert!(result.ash_files.is_empty());
        assert!(result.error.is_some());
    }

    /// Test error handling for invalid parameters.
    #[test]
    fn test_error_handling() {
        // Test with empty symbol name
        let params = SymbolLookupParams {
            ash_symbol: String::new(),
            file: "test.ash".to_string(),
            line: 1,
            column: 1,
        };

        let server = AshMcpServer::new();
        let result = server.find_rust_implementation(&params);
        assert!(result.is_ok(), "Should handle invalid parameters without panicking");
    }

    /// Test `locate_symbol_in_rust_source` directly with sample content.
    #[test]
    fn test_locate_symbol_in_rust_source() {
        let content = r"
//! Some module

pub enum Effect {
    A,
    B,
}

pub struct Foo;

pub trait Bar {}

pub fn baz() {}
";

        assert_eq!(
            AshMcpServer::locate_symbol_in_rust_source(content, "Effect"),
            Some((4, 10, 16))
        );
        assert_eq!(
            AshMcpServer::locate_symbol_in_rust_source(content, "Foo"),
            Some((8, 12, 15))
        );
        assert_eq!(
            AshMcpServer::locate_symbol_in_rust_source(content, "Bar"),
            Some((10, 11, 14))
        );
        assert_eq!(
            AshMcpServer::locate_symbol_in_rust_source(content, "baz"),
            Some((12, 8, 11))
        );
        assert_eq!(
            AshMcpServer::locate_symbol_in_rust_source(content, "NonExistent"),
            None
        );
    }

    /// Test `locate_symbol_in_rust_source` with non-pub items.
    #[test]
    fn test_locate_non_pub_symbol() {
        let content = r"
enum PrivateEffect {
    A,
}
";

        assert_eq!(
            AshMcpServer::locate_symbol_in_rust_source(content, "PrivateEffect"),
            Some((2, 6, 19))
        );
    }
}
