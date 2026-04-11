-- std::test - minimal authored-test helpers (TASK-511)
--
-- Keep this surface intentionally small and parser-stable.
-- The current Ash parser only supports literal panic messages inside function bodies,
-- so these helpers use fixed failure text instead of dynamic formatting.

-- Assert that a boolean value is true.
pub fn assert_true(value: Bool) -> Bool {
    if value then true else panic "assert_true failed"
}

-- Assert that a boolean value is false.
pub fn assert_false(value: Bool) -> Bool {
    if value then panic "assert_false failed" else true
}

-- Assert that two integer values are equal.
pub fn assert_eq_int(expected: Int, actual: Int) -> Bool {
    if expected == actual then true else panic "assert_eq_int failed"
}

-- Assert that two integer values are not equal.
pub fn assert_ne_int(expected: Int, actual: Int) -> Bool {
    if expected != actual then true else panic "assert_ne_int failed"
}

-- Assert that two string values are equal.
pub fn assert_eq_string(expected: String, actual: String) -> Bool {
    if expected == actual then true else panic "assert_eq_string failed"
}

-- Assert that two boolean values are equal.
pub fn assert_eq_bool(expected: Bool, actual: Bool) -> Bool {
    if expected == actual then true else panic "assert_eq_bool failed"
}

-- Explicitly fail the current test.
pub fn fail() -> Bool {
    panic "test failed"
}
