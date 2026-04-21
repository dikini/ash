-- Time capability and functions
--
-- Provides functions for observing current time and sleeping.
-- Observations require the Time capability. Sleep is an execute action.

-- Time capability for time operations
pub capability Time: observe now() returns String
                  | observe now_iso() returns String
                  | observe epoch_millis() returns Int
                  | execute sleep(millis: Int);

-- Get current time as a record with epoch_millis and iso fields
pub fn now() -> String {
    act observe Time.now
}

-- Get current time as ISO 8601 string
pub fn now_iso() -> String {
    act observe Time.now_iso
}

-- Get current time as epoch milliseconds
pub fn epoch_millis() -> Int {
    act observe Time.epoch_millis
}

-- Sleep for the given number of milliseconds
pub fn sleep(millis: Int) {
    act execute Time.sleep with millis: millis;
}
