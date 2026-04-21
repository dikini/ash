//! HTTP capability provider for the Ash engine
//!
//! Provides HTTP client operations:
//! - `get`: HTTP GET request
//! - `post`: HTTP POST with body
//! - `put`: HTTP PUT with body
//! - `delete`: HTTP DELETE request
//! - `head`: HTTP HEAD request (observe)
//!
//! All actions return a structured response record with status, headers, and body.

use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::{Constraint, Effect, Value};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the HTTP provider
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// If set, only requests to these hosts are permitted
    pub allowed_hosts: Option<Vec<String>>,
    /// User-Agent header (default: "Ash/0.1")
    pub user_agent: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            allowed_hosts: None,
            user_agent: "Ash/0.1".to_string(),
        }
    }
}

impl HttpConfig {
    /// Create a new config with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the request timeout in seconds
    #[must_use]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Restrict requests to only these hosts
    #[must_use]
    pub fn with_allowed_hosts(mut self, hosts: Vec<String>) -> Self {
        self.allowed_hosts = Some(hosts);
        self
    }

    /// Set a custom User-Agent header
    #[must_use]
    pub fn with_user_agent(mut self, agent: &str) -> Self {
        self.user_agent = agent.to_string();
        self
    }
}

/// HTTP capability provider
///
/// Implements the unified `CapabilityProvider` trait for HTTP operations.
/// Uses `reqwest` for async HTTP requests.
#[derive(Debug)]
pub struct HttpProvider {
    config: HttpConfig,
    client: reqwest::Client,
}

impl HttpProvider {
    /// Create a new HTTP provider with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(HttpConfig::default())
    }

    /// Create a new HTTP provider with custom configuration
    #[must_use]
    pub fn with_config(config: HttpConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { config, client }
    }

    /// Validate that the given URL's host is in the allowed list
    fn validate_host(&self, url: &str) -> Result<(), CapabilityError> {
        if let Some(ref allowed) = self.config.allowed_hosts {
            let parsed = url::Url::parse(url).map_err(|e| {
                CapabilityError::InvalidArgument(format!("Invalid URL '{url}': {e}"))
            })?;
            let host = parsed
                .host_str()
                .ok_or_else(|| {
                    CapabilityError::InvalidArgument(format!("URL has no host: {url}"))
                })?;
            if !allowed.iter().any(|h| h == host) {
                return Err(CapabilityError::PermissionDenied(format!(
                    "Host '{host}' not in allowed list"
                )));
            }
        }
        Ok(())
    }

    /// Extract a URL string from the first argument
    fn extract_url(args: &[Value]) -> Result<String, CapabilityError> {
        match args.first() {
            Some(Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(CapabilityError::InvalidArgument(
                "URL must be a string".to_string(),
            )),
            None => Err(CapabilityError::InvalidArgument(
                "Missing URL argument".to_string(),
            )),
        }
    }

    /// Extract a body string from the second argument
    fn extract_body(args: &[Value]) -> Result<String, CapabilityError> {
        match args.get(1) {
            Some(Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(CapabilityError::InvalidArgument(
                "Body must be a string".to_string(),
            )),
            None => Err(CapabilityError::InvalidArgument(
                "Missing body argument".to_string(),
            )),
        }
    }

    /// Extract optional headers from a Record value (third argument)
    fn extract_headers(args: &[Value]) -> Result<HashMap<String, String>, CapabilityError> {
        match args.get(2) {
            Some(Value::Record(fields)) => {
                let mut headers = HashMap::new();
                for (key, val) in fields.iter() {
                    match val {
                        Value::String(s) => {
                            headers.insert(key.clone(), s.clone());
                        }
                        _ => {
                            return Err(CapabilityError::InvalidArgument(format!(
                                "Header value for '{key}' must be a string"
                            )));
                        }
                    }
                }
                Ok(headers)
            }
            Some(_) => Err(CapabilityError::InvalidArgument(
                "Headers must be a record".to_string(),
            )),
            None => Ok(HashMap::new()),
        }
    }

    /// Build a request with optional headers
    fn apply_headers(
        builder: reqwest::RequestBuilder,
        headers: &HashMap<String, String>,
    ) -> reqwest::RequestBuilder {
        let mut builder = builder;
        for (key, value) in headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
        builder
    }

    /// Convert a reqwest Response into an Ash Value::Record
    async fn response_to_value(
        response: reqwest::Response,
    ) -> Result<Value, CapabilityError> {
        let status = response.status().as_u16() as i64;

        let headers: HashMap<String, Value> = response
            .headers()
            .iter()
            .map(|(name, val)| {
                (
                    name.to_string(),
                    Value::String(
                        val.to_str()
                            .unwrap_or("<non-ascii>")
                            .to_string(),
                    ),
                )
            })
            .collect();

        let body = response
            .text()
            .await
            .map_err(|e| CapabilityError::ExecutionFailed(format!("Failed to read response body: {e}")))?;

        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::Int(status));
        result.insert("headers".to_string(), Value::Record(Box::new(headers)));
        result.insert("body".to_string(), Value::String(body));

        Ok(Value::Record(Box::new(result)))
    }

    /// Perform a GET request
    async fn do_get(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let url = Self::extract_url(args)?;
        self.validate_host(&url)?;
        let headers = Self::extract_headers(args)?;

        let builder = Self::apply_headers(self.client.get(&url), &headers);
        let response = builder.send().await.map_err(|e| {
            CapabilityError::ExecutionFailed(format!("HTTP GET failed: {e}"))
        })?;

        Self::response_to_value(response).await
    }

    /// Perform a POST request
    async fn do_post(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let url = Self::extract_url(args)?;
        self.validate_host(&url)?;
        let body = Self::extract_body(args)?;
        let headers = Self::extract_headers(args)?;

        let builder = Self::apply_headers(
            self.client.post(&url).body(body),
            &headers,
        );
        let response = builder.send().await.map_err(|e| {
            CapabilityError::ExecutionFailed(format!("HTTP POST failed: {e}"))
        })?;

        Self::response_to_value(response).await
    }

    /// Perform a PUT request
    async fn do_put(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let url = Self::extract_url(args)?;
        self.validate_host(&url)?;
        let body = Self::extract_body(args)?;
        let headers = Self::extract_headers(args)?;

        let builder = Self::apply_headers(
            self.client.put(&url).body(body),
            &headers,
        );
        let response = builder.send().await.map_err(|e| {
            CapabilityError::ExecutionFailed(format!("HTTP PUT failed: {e}"))
        })?;

        Self::response_to_value(response).await
    }

    /// Perform a DELETE request
    async fn do_delete(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let url = Self::extract_url(args)?;
        self.validate_host(&url)?;
        let headers = Self::extract_headers(args)?;

        let builder = Self::apply_headers(self.client.delete(&url), &headers);
        let response = builder.send().await.map_err(|e| {
            CapabilityError::ExecutionFailed(format!("HTTP DELETE failed: {e}"))
        })?;

        Self::response_to_value(response).await
    }

    /// Perform a HEAD request (observe)
    async fn do_head(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let url = Self::extract_url(args)?;
        self.validate_host(&url)?;

        let response = self.client.head(&url).send().await.map_err(|e| {
            CapabilityError::ExecutionFailed(format!("HTTP HEAD failed: {e}"))
        })?;

        let status = response.status().as_u16() as i64;
        let headers: HashMap<String, Value> = response
            .headers()
            .iter()
            .map(|(name, val)| {
                (
                    name.to_string(),
                    Value::String(
                        val.to_str()
                            .unwrap_or("<non-ascii>")
                            .to_string(),
                    ),
                )
            })
            .collect();

        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::Int(status));
        result.insert("headers".to_string(), Value::Record(Box::new(headers)));

        Ok(Value::Record(Box::new(result)))
    }
}

impl Default for HttpProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for HttpProvider {
    fn name(&self) -> &str {
        "http"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        if constraints.is_empty() {
            return Err(CapabilityError::InvalidArgument(
                "No observe constraints provided".to_string(),
            ));
        }
        let action_name = constraints[0].predicate.name.as_str();
        match action_name {
            "head" => {
                // Extract URL from constraint arguments
                // Constraints hold unevaluated Expr; observe path not yet supported
                let _args: Vec<Value> = constraints[0]
                    .predicate
                    .arguments
                    .iter()
                    .map(|_| Value::Null)
                    .collect();
                Err(CapabilityError::NotAvailable(
                    "HTTP observe requires execute path. Use execute(\"head\", args) instead."
                        .to_string(),
                ))
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown HTTP observe action: {action_name}"
            ))),
        }
    }

    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        match action_name {
            "get" => self.do_get(args).await,
            "post" => self.do_post(args).await,
            "put" => self.do_put(args).await,
            "delete" => self.do_delete(args).await,
            "head" => self.do_head(args).await,
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown HTTP action: {action_name}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_provider_name() {
        let provider = HttpProvider::new();
        assert_eq!(provider.name(), "http");
    }

    #[test]
    fn test_http_provider_effect_is_operational() {
        let provider = HttpProvider::new();
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert!(config.allowed_hosts.is_none());
        assert_eq!(config.user_agent, "Ash/0.1");
    }

    #[test]
    fn test_http_config_builder() {
        let config = HttpConfig::new()
            .with_timeout(60)
            .with_allowed_hosts(vec!["api.example.com".to_string()])
            .with_user_agent("MyApp/1.0");
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(
            config.allowed_hosts,
            Some(vec!["api.example.com".to_string()])
        );
        assert_eq!(config.user_agent, "MyApp/1.0");
    }

    #[test]
    fn test_extract_url_valid() {
        let args = [Value::String("https://example.com".to_string())];
        let url = HttpProvider::extract_url(&args).unwrap();
        assert_eq!(url, "https://example.com");
    }

    #[test]
    fn test_extract_url_missing() {
        let args: Vec<Value> = vec![];
        let err = HttpProvider::extract_url(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_url_wrong_type() {
        let args = [Value::Int(42)];
        let err = HttpProvider::extract_url(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_body_valid() {
        let args = [
            Value::String("https://example.com".to_string()),
            Value::String("{\"key\":\"value\"}".to_string()),
        ];
        let body = HttpProvider::extract_body(&args).unwrap();
        assert_eq!(body, "{\"key\":\"value\"}");
    }

    #[test]
    fn test_extract_body_missing() {
        let args = [Value::String("https://example.com".to_string())];
        let err = HttpProvider::extract_body(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_headers_from_record() {
        let mut fields = HashMap::new();
        fields.insert("Content-Type".to_string(), Value::String("application/json".to_string()));
        fields.insert("Accept".to_string(), Value::String("text/html".to_string()));
        let args = [
            Value::String("https://example.com".to_string()),
            Value::String("body".to_string()),
            Value::Record(Box::new(fields)),
        ];
        let headers = HttpProvider::extract_headers(&args).unwrap();
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("Accept").unwrap(), "text/html");
    }

    #[test]
    fn test_extract_headers_missing_uses_empty() {
        let args = [
            Value::String("https://example.com".to_string()),
            Value::String("body".to_string()),
        ];
        let headers = HttpProvider::extract_headers(&args).unwrap();
        assert!(headers.is_empty());
    }

    #[test]
    fn test_extract_headers_wrong_type() {
        let args = [
            Value::String("https://example.com".to_string()),
            Value::String("body".to_string()),
            Value::Int(42),
        ];
        let err = HttpProvider::extract_headers(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_validate_host_allowed() {
        let config = HttpConfig::new()
            .with_allowed_hosts(vec!["example.com".to_string()]);
        let provider = HttpProvider::with_config(config);

        // Should succeed
        provider.validate_host("https://example.com/path").unwrap();
    }

    #[test]
    fn test_validate_host_blocked() {
        let config = HttpConfig::new()
            .with_allowed_hosts(vec!["example.com".to_string()]);
        let provider = HttpProvider::with_config(config);

        // Should fail
        let err = provider
            .validate_host("https://evil.com/path")
            .unwrap_err();
        assert!(matches!(err, CapabilityError::PermissionDenied(_)));
    }

    #[test]
    fn test_validate_host_no_restriction() {
        let provider = HttpProvider::new();
        // No allowed_hosts set, should succeed for any URL
        provider.validate_host("https://any-host.com/path").unwrap();
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        let provider = HttpProvider::new();
        let err = provider
            .execute("patch", &[Value::String("https://example.com".to_string())])
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_get_missing_url() {
        let provider = HttpProvider::new();
        let err = provider.execute("get", &[]).await.unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_post_missing_body() {
        let provider = HttpProvider::new();
        let err = provider
            .execute("post", &[Value::String("https://example.com".to_string())])
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_get_blocked_host() {
        let config = HttpConfig::new()
            .with_allowed_hosts(vec!["allowed.com".to_string()]);
        let provider = HttpProvider::with_config(config);
        let err = provider
            .execute("get", &[Value::String("https://blocked.com/path".to_string())])
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::PermissionDenied(_)));
    }

    #[tokio::test]
    async fn test_observe_empty_constraints() {
        let provider = HttpProvider::new();
        let err = provider.observe(&[]).await.unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_observe_unknown_action() {
        let provider = HttpProvider::new();
        let predicate = ash_core::ast::Predicate {
            name: "unknown".into(),
            arguments: vec![],
        };
        let constraint = Constraint { predicate };
        let err = provider.observe(&[constraint]).await.unwrap_err();
        assert!(matches!(err, CapabilityError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_get_invalid_url() {
        let provider = HttpProvider::new();
        let err = provider
            .execute("get", &[Value::String("not-a-url".to_string())])
            .await
            .unwrap_err();
        // reqwest will fail to connect
        assert!(matches!(err, CapabilityError::ExecutionFailed(_)));
    }
}
