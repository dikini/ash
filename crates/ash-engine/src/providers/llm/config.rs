//! LLM Provider Configuration
//!
//! Defines `LlmConfig` for per-provider connection settings, with validation and defaults.

use std::fmt;

/// Configuration for an LLM provider connection
///
/// This struct holds all settings needed to connect to an OpenAI-compatible API endpoint.
/// The `api_key` field is redacted in both `Debug` and `Display` output for security.
#[derive(Clone)]
pub struct LlmConfig {
    /// API base URL (e.g., "<https://api.openai.com/v1>")
    pub api_base: String,
    /// API key for authentication (redacted in Debug output)
    pub api_key: String,
    /// Default model identifier if none specified
    pub default_model: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum retry count for transient failures
    pub max_retries: u32,
}

impl Default for LlmConfig {
    /// Default configuration for `OpenAI` API
    ///
    /// Uses:
    /// - `api_base`: "<https://api.openai.com/v1>"
    /// - `api_key`: empty string (must be set before use)
    /// - `default_model`: "gpt-4o"
    /// - `timeout_ms`: 30000 (30 seconds)
    /// - `max_retries`: 2
    fn default() -> Self {
        Self {
            api_base: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            default_model: "gpt-4o".to_string(),
            timeout_ms: 30000,
            max_retries: 2,
        }
    }
}

impl LlmConfig {
    /// Validate the configuration
    ///
    /// Checks:
    /// - `api_base` must be a valid URL with http or https scheme
    /// - `api_key` must not be empty (unless explicitly allowed by caller)
    ///
    /// # Errors
    /// Returns `Err(String)` with a description of the validation failure
    pub fn validate(&self) -> Result<(), String> {
        // Validate api_base is a valid URL
        match url::Url::parse(&self.api_base) {
            Ok(url) => {
                let scheme = url.scheme();
                if scheme != "http" && scheme != "https" {
                    return Err(format!(
                        "api_base must use http or https scheme, got: {scheme}"
                    ));
                }
            }
            Err(e) => {
                return Err(format!("api_base is not a valid URL: {e}"));
            }
        }

        // Validate api_key is not empty
        if self.api_key.is_empty() {
            return Err("api_key must not be empty".to_string());
        }

        Ok(())
    }

    /// Validate configuration allowing empty API key for local providers
    ///
    /// This is useful for local providers like Ollama that don't require authentication.
    ///
    /// # Errors
    /// Returns `Err(String)` with a description of the validation failure
    pub fn validate_for_local(&self) -> Result<(), String> {
        // Validate api_base is a valid URL
        match url::Url::parse(&self.api_base) {
            Ok(url) => {
                let scheme = url.scheme();
                if scheme != "http" && scheme != "https" {
                    return Err(format!(
                        "api_base must use http or https scheme, got: {scheme}"
                    ));
                }
            }
            Err(e) => {
                return Err(format!("api_base is not a valid URL: {e}"));
            }
        }

        // api_key can be empty for local providers
        Ok(())
    }

    /// Create a new config for `OpenAI` API
    ///
    /// # Arguments
    /// * `api_key` - Your `OpenAI` API key
    #[must_use]
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            ..Self::default()
        }
    }

    /// Create a new config for a local Ollama instance
    ///
    /// Uses `http://localhost:11434` as the base URL and no API key.
    #[must_use]
    pub fn ollama() -> Self {
        Self {
            api_base: "http://localhost:11434/v1".to_string(),
            api_key: String::new(),
            default_model: "llama3.2".to_string(),
            timeout_ms: 60000,
            max_retries: 1,
        }
    }

    /// Create a new config with custom base URL
    ///
    /// # Arguments
    /// * `api_base` - Custom API base URL
    /// * `api_key` - API key (can be empty for local providers)
    #[must_use]
    pub fn custom(api_base: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            api_base: api_base.into(),
            api_key: api_key.into(),
            ..Self::default()
        }
    }
}

impl fmt::Display for LlmConfig {
    /// Display the config with API key redacted
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LlmConfig {{ api_base: {}, api_key: ***, default_model: {}, timeout_ms: {}, max_retries: {} }}",
            self.api_base, self.default_model, self.timeout_ms, self.max_retries
        )
    }
}

impl fmt::Debug for LlmConfig {
    /// Debug the config with API key redacted
    ///
    /// The `api_key` field is always shown as `"***"` to prevent accidental
    /// secret leakage in logs and error messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LlmConfig")
            .field("api_base", &self.api_base)
            .field("api_key", &"***")
            .field("default_model", &self.default_model)
            .field("timeout_ms", &self.timeout_ms)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.api_base, "https://api.openai.com/v1");
        assert_eq!(config.api_key, "");
        assert_eq!(config.default_model, "gpt-4o");
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_retries, 2);
    }

    #[test]
    fn test_validate_accepts_valid_config() {
        let config = LlmConfig {
            api_base: "https://api.openai.com/v1".to_string(),
            api_key: "sk-test123".to_string(),
            default_model: "gpt-4o".to_string(),
            timeout_ms: 30000,
            max_retries: 2,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_invalid_url() {
        let config = LlmConfig {
            api_base: "not-a-valid-url".to_string(),
            api_key: "sk-test123".to_string(),
            ..LlmConfig::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a valid URL"));
    }

    #[test]
    fn test_validate_rejects_empty_api_key() {
        let config = LlmConfig {
            api_base: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            ..LlmConfig::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "api_key must not be empty");
    }

    #[test]
    fn test_validate_rejects_non_http_scheme() {
        let config = LlmConfig {
            api_base: "ftp://example.com".to_string(),
            api_key: "sk-test123".to_string(),
            ..LlmConfig::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must use http or https"));
    }

    #[test]
    fn test_validate_for_local_allows_empty_key() {
        let config = LlmConfig::ollama();
        assert!(config.validate_for_local().is_ok());
    }

    #[test]
    fn test_openai_factory() {
        let config = LlmConfig::openai("sk-mykey");
        assert_eq!(config.api_key, "sk-mykey");
        assert_eq!(config.api_base, "https://api.openai.com/v1");
    }

    #[test]
    fn test_ollama_factory() {
        let config = LlmConfig::ollama();
        assert_eq!(config.api_base, "http://localhost:11434/v1");
        assert_eq!(config.api_key, "");
        assert_eq!(config.default_model, "llama3.2");
    }

    #[test]
    fn test_custom_factory() {
        let config = LlmConfig::custom("https://custom.example.com", "custom-key");
        assert_eq!(config.api_base, "https://custom.example.com");
        assert_eq!(config.api_key, "custom-key");
    }

    #[test]
    fn test_display_redacts_api_key() {
        let config = LlmConfig::openai("sk-secret");
        let display = format!("{config}");
        assert!(display.contains("api_key: ***"));
        assert!(!display.contains("sk-secret"));
    }

    #[test]
    fn test_debug_redacts_api_key() {
        let secret_key = "sk-test-real-api-key-12345";
        let config = LlmConfig::openai(secret_key);
        let debug = format!("{config:?}");
        // Debug should NOT contain the raw secret
        assert!(
            !debug.contains(secret_key),
            "Debug output leaked api_key: {debug}"
        );
        // Debug should show redacted marker
        assert!(
            debug.contains("***"),
            "Debug output missing redaction marker: {debug}"
        );
        // Debug should contain other fields
        assert!(debug.contains("api_base"));
        assert!(debug.contains("default_model"));
    }

    #[test]
    fn test_ollama_config_fails_strict_validation() {
        // Ollama config has empty api_key by design
        let config = LlmConfig::ollama();
        assert!(config.api_key.is_empty());
        // Strict validate() should reject empty api_key
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "api_key must not be empty");
    }
}
