//! Time capability provider for the Ash engine
//!
//! Provides time-related capability operations:
//! - `now`: Observe current time as epoch millis
//! - `now_iso`: Observe current time as ISO 8601 string
//! - `sleep`: Execute a sleep/delay for N milliseconds

use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::{Constraint, Effect, Value};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Time capability provider
///
/// Implements the unified `CapabilityProvider` trait for time operations.
/// All time operations are side-effect-free reads except `sleep` which
/// delays execution.
#[derive(Debug, Clone)]
pub struct TimeProvider {
    /// If set, overrides system time for deterministic testing (epoch millis)
    mock_now: Option<u64>,
}

impl TimeProvider {
    /// Create a new time provider using real system time
    #[must_use]
    pub fn new() -> Self {
        Self { mock_now: None }
    }

    /// Create a time provider with a fixed mock time for testing
    #[must_use]
    pub fn mock(epoch_millis: u64) -> Self {
        Self {
            mock_now: Some(epoch_millis),
        }
    }

    /// Get current epoch millis
    fn current_millis(&self) -> u64 {
        self.mock_now.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        })
    }

    /// Get current time as ISO 8601 string
    fn current_iso(&self) -> String {
        let millis = self.current_millis();
        let secs = millis / 1000;
        let subsec_millis = millis % 1000;
        // Calculate date components from epoch seconds
        let days = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        // Convert days since epoch to year/month/day
        let (year, month, day) = days_to_date(days);

        format!(
            "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}.{subsec_millis:03}Z"
        )
    }

    /// Extract milliseconds from a Value argument
    fn extract_millis(args: &[Value]) -> Result<u64, CapabilityError> {
        match args.first() {
            Some(Value::Int(n)) => {
                if *n < 0 {
                    return Err(CapabilityError::InvalidArgument(
                        "Sleep duration must be non-negative".to_string(),
                    ));
                }
                Ok(*n as u64)
            }
            Some(_) => Err(CapabilityError::InvalidArgument(
                "Duration must be an integer (milliseconds)".to_string(),
            )),
            None => Err(CapabilityError::InvalidArgument(
                "Missing duration argument".to_string(),
            )),
        }
    }

    /// Handle `now` observe action
    fn handle_now(&self) -> Result<Value, CapabilityError> {
        let millis = self.current_millis();
        let mut result = HashMap::new();
        result.insert("epoch_millis".to_string(), Value::Int(millis as i64));
        result.insert("iso".to_string(), Value::String(self.current_iso()));
        Ok(Value::Record(Box::new(result)))
    }

    /// Handle `now_iso` observe action
    fn handle_now_iso(&self) -> Result<Value, CapabilityError> {
        Ok(Value::String(self.current_iso()))
    }

    /// Handle `sleep` execute action
    async fn handle_sleep(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let millis = Self::extract_millis(args)?;
        tokio::time::sleep(Duration::from_millis(millis)).await;
        Ok(Value::Null)
    }

    /// Handle `epoch_millis` observe action (returns just the integer)
    fn handle_epoch_millis(&self) -> Result<Value, CapabilityError> {
        Ok(Value::Int(self.current_millis() as i64))
    }
}

impl Default for TimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert days since Unix epoch to (year, month, day)
fn days_to_date(days_since_epoch: u64) -> (i64, u32, u32) {
    // 1970-01-01 is day 0
    let mut year = 1970_i64;
    let mut remaining = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0_u32;
    for &days in &month_days {
        if remaining < days {
            break;
        }
        remaining -= days;
        month += 1;
    }

    (year, month + 1, remaining as u32 + 1)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[async_trait]
impl CapabilityProvider for TimeProvider {
    fn name(&self) -> &str {
        "time"
    }

    fn effect(&self) -> Effect {
        // Time observation is Epistemic, but sleep is Operational.
        // We report Deliberative as the safe middle ground.
        // Sleep requires the Operational path through execute().
        Effect::Deliberative
    }

    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        if constraints.is_empty() {
            return Err(CapabilityError::InvalidArgument(
                "No observe constraints provided".to_string(),
            ));
        }
        let action_name = constraints[0].predicate.name.as_str();
        match action_name {
            "now" => self.handle_now(),
            "now_iso" => self.handle_now_iso(),
            "epoch_millis" => self.handle_epoch_millis(),
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown time observe action: {action_name}"
            ))),
        }
    }

    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        match action_name {
            "sleep" => self.handle_sleep(args).await,
            "now" => self.handle_now(),
            "now_iso" => self.handle_now_iso(),
            "epoch_millis" => self.handle_epoch_millis(),
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown time action: {action_name}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_provider_name() {
        let provider = TimeProvider::new();
        assert_eq!(provider.name(), "time");
    }

    #[test]
    fn test_time_provider_effect() {
        let provider = TimeProvider::new();
        assert_eq!(provider.effect(), Effect::Deliberative);
    }

    #[test]
    fn test_mock_now_returns_fixed_time() {
        let provider = TimeProvider::mock(1700000000000);
        assert_eq!(provider.current_millis(), 1700000000000);
    }

    #[test]
    fn test_new_uses_system_time() {
        let provider = TimeProvider::new();
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let now = provider.current_millis();
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        assert!(now >= before && now <= after);
    }

    #[test]
    fn test_handle_now_returns_record() {
        let provider = TimeProvider::mock(1700000000000);
        let result = provider.handle_now().unwrap();
        match result {
            Value::Record(fields) => {
                assert!(fields.contains_key("epoch_millis"));
                assert!(fields.contains_key("iso"));
                assert_eq!(fields.get("epoch_millis"), Some(&Value::Int(1700000000000)));
            }
            _ => panic!("Expected Record, got {result:?}"),
        }
    }

    #[test]
    fn test_handle_now_iso_returns_string() {
        let provider = TimeProvider::mock(1700000000000);
        let result = provider.handle_now_iso().unwrap();
        match result {
            Value::String(s) => {
                assert!(s.starts_with("2023"));
                assert!(s.ends_with('Z'));
                assert!(s.contains('T'));
            }
            _ => panic!("Expected String, got {result:?}"),
        }
    }

    #[test]
    fn test_handle_epoch_millis_returns_int() {
        let provider = TimeProvider::mock(1700000000000);
        let result = provider.handle_epoch_millis().unwrap();
        assert_eq!(result, Value::Int(1700000000000));
    }

    #[test]
    fn test_extract_millis_valid() {
        let args = [Value::Int(1000)];
        assert_eq!(TimeProvider::extract_millis(&args).unwrap(), 1000);
    }

    #[test]
    fn test_extract_millis_zero() {
        let args = [Value::Int(0)];
        assert_eq!(TimeProvider::extract_millis(&args).unwrap(), 0);
    }

    #[test]
    fn test_extract_millis_negative() {
        let args = [Value::Int(-1)];
        let err = TimeProvider::extract_millis(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_millis_wrong_type() {
        let args = [Value::String("1000".to_string())];
        let err = TimeProvider::extract_millis(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_millis_missing() {
        let args: Vec<Value> = vec![];
        let err = TimeProvider::extract_millis(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_sleep_executes() {
        let provider = TimeProvider::new();
        let result = provider
            .execute("sleep", &[Value::Int(1)])
            .await
            .unwrap();
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        let provider = TimeProvider::new();
        let err = provider
            .execute("unknown", &[])
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_observe_empty_constraints() {
        let provider = TimeProvider::new();
        let err = provider.observe(&[]).await.unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_days_to_date_epoch() {
        assert_eq!(days_to_date(0), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_date_known_date() {
        // 2023-01-01 is 19358 days since epoch
        assert_eq!(days_to_date(19358), (2023, 1, 1));
    }

    #[test]
    fn test_days_to_date_leap_year() {
        // 2024-02-29 is 19782 days since epoch
        assert_eq!(days_to_date(19782), (2024, 2, 29));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
    }

    #[test]
    fn test_mock_iso_format() {
        let provider = TimeProvider::mock(0);
        let iso = provider.current_iso();
        assert_eq!(iso, "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_clone_preserves_mock() {
        let provider = TimeProvider::mock(1700000000000);
        let cloned = provider.clone();
        assert_eq!(cloned.current_millis(), 1700000000000);
    }
}
