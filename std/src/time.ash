-- Time capability and functions
--
-- Provides parser-checkable runtime-provided declarations for observing current
-- time and sleeping. The Time capability below records the intended authority
-- contract; concrete capability-wrapper bodies remain deferred until the
-- parser/runtime support a canonical stdlib `act` wrapper spelling.

-- Time.now returns a runtime record with epoch_millis: Int and iso: String.
pub capability Time: observe now() returns Record
                  | observe now_iso() returns String
                  | observe epoch_millis() returns Int
                  | execute sleep(millis: Int);

-- Get current time as the current runtime-provider record representation
pub builtin fn now() -> Record;

-- Get current time as ISO 8601 string
pub builtin fn now_iso() -> String;

-- Get current time as epoch milliseconds
pub builtin fn epoch_millis() -> Int;

-- Sleep for the given number of milliseconds
pub builtin fn sleep(millis: Int) -> Unit;
