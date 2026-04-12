//! Shared error mapping for OpenAI-compatible API errors
//!
//! Centralises the mapping from `async_openai::error::OpenAIError` to
//! `CapabilityError` so that both the models and provider modules share
//! one implementation.
//!
//! Error mapping from SPEC-029 §9.4:
//! - 401/403 -> `PermissionDenied("auth_error")`
//! - 404 -> `NotAvailable("model_not_found")`
//! - 429 -> `ExecutionFailed("rate_limited")`
//! - Connection errors -> `ExecutionFailed("network_error")`
//! - 400 -> `InvalidArgument("invalid_request")`
//! - 5xx -> `ExecutionFailed("server_error")`

use ash_core::capability::CapabilityError;
use async_openai::error::OpenAIError;

/// Map an [`OpenAIError`] into a [`CapabilityError`].
///
/// `context` is an optional label (e.g. provider name or `"models"`) that is
/// included in fallback error messages to aid debugging.
///
/// # Errors
///
/// This is a pure transformation function -- it never fails, but the returned
/// [`CapabilityError`] variant reflects the category of the upstream error.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn map_openai_error(error: OpenAIError, context: &str) -> CapabilityError {
    match &error {
        OpenAIError::ApiError(api_err) => {
            let code = api_err.code.as_deref().unwrap_or("");
            let error_type = api_err.r#type.as_deref().unwrap_or("");

            match code {
                "invalid_api_key" | "authentication_error" => {
                    CapabilityError::PermissionDenied("auth_error".to_string())
                }
                "rate_limit_exceeded" => {
                    CapabilityError::ExecutionFailed("rate_limited".to_string())
                }
                "model_not_found" | "not_found" => {
                    CapabilityError::NotAvailable("model_not_found".to_string())
                }
                "invalid_request_error" => {
                    CapabilityError::InvalidArgument("invalid_request".to_string())
                }
                _ => match error_type {
                    "authentication_error" | "permission_error" => {
                        CapabilityError::PermissionDenied("auth_error".to_string())
                    }
                    "rate_limit_error" => {
                        CapabilityError::ExecutionFailed("rate_limited".to_string())
                    }
                    "not_found_error" => {
                        CapabilityError::NotAvailable("model_not_found".to_string())
                    }
                    "invalid_request_error" => {
                        CapabilityError::InvalidArgument("invalid_request".to_string())
                    }
                    "server_error" => CapabilityError::ExecutionFailed("server_error".to_string()),
                    _ => CapabilityError::ExecutionFailed(format!("api_error[{context}]: {code}")),
                },
            }
        }
        OpenAIError::Reqwest(e) => {
            if e.is_connect() || e.is_timeout() {
                CapabilityError::ExecutionFailed("network_error".to_string())
            } else {
                CapabilityError::ExecutionFailed(format!("request_error[{context}]: {e}"))
            }
        }
        OpenAIError::JSONDeserialize(e) => {
            CapabilityError::ExecutionFailed(format!("deserialize_error: {e}"))
        }
        _ => CapabilityError::ExecutionFailed(format!("openai_error[{context}]: {error}")),
    }
}
