//! Regex capability provider for the Ash engine
//!
//! Provides regex operations backed by the `regex` Rust crate:
//! - `find`: Find the first match of a pattern in text
//! - `matches`: Check if a pattern matches anywhere in text
//! - `replace`: Replace all matches of a pattern in text

use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::{Constraint, Effect, Value};
use async_trait::async_trait;

/// Regex capability provider
///
/// Provides regex operations:
/// - `find`: Find the first match of a pattern in text, returns `Option<String>`
/// - `matches`: Check if a pattern matches anywhere in text, returns `Bool`
/// - `replace`: Replace all matches of a pattern with a replacement string, returns `String`
///
/// On invalid pattern, returns `CapabilityError::InvalidArgument`.
#[derive(Debug, Clone)]
pub struct RegexProvider;

impl RegexProvider {
    /// Create a new regex provider
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extract a string from a Value argument
    fn extract_string(arg: &Value) -> Result<String, CapabilityError> {
        match arg {
            Value::String(s) => Ok(s.clone()),
            _ => Err(CapabilityError::InvalidArgument(
                "Argument must be a string".to_string(),
            )),
        }
    }

    /// Handle `find` execute operation
    fn handle_find(pattern: &str, text: &str) -> Result<Value, CapabilityError> {
        let re = regex::Regex::new(pattern)
            .map_err(|e| CapabilityError::InvalidArgument(format!("Invalid regex pattern: {e}")))?;

        if let Some(mat) = re.find(text) {
            Ok(Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![(
                    "value".to_string(),
                    Value::String(mat.as_str().to_string()),
                )]),
            })
        } else {
            Ok(Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            })
        }
    }

    /// Handle `matches` execute operation
    fn handle_matches(pattern: &str, text: &str) -> Result<Value, CapabilityError> {
        let re = regex::Regex::new(pattern)
            .map_err(|e| CapabilityError::InvalidArgument(format!("Invalid regex pattern: {e}")))?;

        Ok(Value::Bool(re.is_match(text)))
    }

    /// Handle `replace` execute operation
    fn handle_replace(
        pattern: &str,
        replacement: &str,
        text: &str,
    ) -> Result<Value, CapabilityError> {
        let re = regex::Regex::new(pattern)
            .map_err(|e| CapabilityError::InvalidArgument(format!("Invalid regex pattern: {e}")))?;

        Ok(Value::String(re.replace_all(text, replacement).to_string()))
    }
}

impl Default for RegexProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for RegexProvider {
    fn name(&self) -> &'static str {
        "regex"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Err(CapabilityError::NotAvailable(
            "Regex provider does not support observe operations".to_string(),
        ))
    }

    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        match action_name {
            "find" => {
                if args.len() < 2 {
                    return Err(CapabilityError::InvalidArgument(
                        "find requires pattern and text arguments".to_string(),
                    ));
                }
                let pattern = Self::extract_string(&args[0])?;
                let text = Self::extract_string(&args[1])?;
                Self::handle_find(&pattern, &text)
            }
            "matches" => {
                if args.len() < 2 {
                    return Err(CapabilityError::InvalidArgument(
                        "matches requires pattern and text arguments".to_string(),
                    ));
                }
                let pattern = Self::extract_string(&args[0])?;
                let text = Self::extract_string(&args[1])?;
                Self::handle_matches(&pattern, &text)
            }
            "replace" => {
                if args.len() < 3 {
                    return Err(CapabilityError::InvalidArgument(
                        "replace requires pattern, replacement, and text arguments".to_string(),
                    ));
                }
                let pattern = Self::extract_string(&args[0])?;
                let replacement = Self::extract_string(&args[1])?;
                let text = Self::extract_string(&args[2])?;
                Self::handle_replace(&pattern, &replacement, &text)
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown execute action: {action_name}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_provider_new() {
        let provider = RegexProvider::new();
        let _ = provider;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_regex_provider_default() {
        let provider: RegexProvider = RegexProvider::default();
        let _ = provider;
    }

    #[test]
    fn test_regex_provider_name() {
        let provider = RegexProvider::new();
        assert_eq!(provider.name(), "regex");
    }

    #[test]
    fn test_regex_provider_effect() {
        assert_eq!(RegexProvider::new().effect(), Effect::Operational);
    }

    #[test]
    fn test_regex_extract_string() {
        let value = Value::String("hello".to_string());
        assert_eq!(RegexProvider::extract_string(&value).unwrap(), "hello");
    }

    #[test]
    fn test_regex_extract_string_invalid() {
        let value = Value::Int(42);
        assert!(RegexProvider::extract_string(&value).is_err());
    }

    #[tokio::test]
    async fn test_regex_provider_find_success() {
        let provider = RegexProvider::new();
        let result = provider
            .execute(
                "find",
                &[
                    Value::String(r"\d+".to_string()),
                    Value::String("abc123def".to_string()),
                ],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::Variant {
                name: "Some".to_string(),
                fields: Box::new(vec![(
                    "value".to_string(),
                    Value::String("123".to_string())
                )]),
            }
        );
    }

    #[tokio::test]
    async fn test_regex_provider_find_none() {
        let provider = RegexProvider::new();
        let result = provider
            .execute(
                "find",
                &[
                    Value::String(r"\d+".to_string()),
                    Value::String("abcdef".to_string()),
                ],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }
        );
    }

    #[tokio::test]
    async fn test_regex_provider_matches_true() {
        let provider = RegexProvider::new();
        let result = provider
            .execute(
                "matches",
                &[
                    Value::String(r"hello".to_string()),
                    Value::String("hello world".to_string()),
                ],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[tokio::test]
    async fn test_regex_provider_matches_false() {
        let provider = RegexProvider::new();
        let result = provider
            .execute(
                "matches",
                &[
                    Value::String(r"goodbye".to_string()),
                    Value::String("hello world".to_string()),
                ],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[tokio::test]
    async fn test_regex_provider_replace() {
        let provider = RegexProvider::new();
        let result = provider
            .execute(
                "replace",
                &[
                    Value::String(r"\d+".to_string()),
                    Value::String("NUM".to_string()),
                    Value::String("abc123def".to_string()),
                ],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("abcNUMdef".to_string()));
    }

    #[tokio::test]
    async fn test_regex_provider_invalid_pattern() {
        let provider = RegexProvider::new();
        let result = provider
            .execute(
                "find",
                &[
                    Value::String(r"[invalid".to_string()),
                    Value::String("text".to_string()),
                ],
            )
            .await;
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), CapabilityError::InvalidArgument(_)),
            "Expected InvalidArgument error for invalid pattern"
        );
    }

    #[tokio::test]
    async fn test_regex_provider_unknown_action() {
        let provider = RegexProvider::new();
        let result = provider.execute("unknown", &[]).await;
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), CapabilityError::NotAvailable(_)),
            "Expected NotAvailable error for unknown action"
        );
    }

    #[tokio::test]
    async fn test_regex_provider_insufficient_args() {
        let provider = RegexProvider::new();
        let result = provider
            .execute("find", &[Value::String(r"\d+".to_string())])
            .await;
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), CapabilityError::InvalidArgument(_)),
            "Expected InvalidArgument error for insufficient args"
        );
    }
}
