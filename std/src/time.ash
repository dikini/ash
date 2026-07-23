-- Time capability and functions
--
-- Provides runtime-backed declarations for observing current time and sleeping.
-- The Time capability records the intended authority contract; Phase 198
-- profiles admit explicit clock rows and support deterministic test-clock
-- provider installation for repeatable evidence.

-- Time.now returns a runtime record with epoch_millis: Int and iso: String.
-- Get current time as the current runtime-provider record representation
pub builtin fn now() -> { epoch_millis: Int, iso: String };

-- Get current time as ISO 8601 string
pub builtin fn now_iso() -> String;

-- Get current time as epoch milliseconds
pub builtin fn epoch_millis() -> Int;

-- Sleep for the given number of milliseconds
pub builtin fn sleep(millis: Int) -> Unit;
