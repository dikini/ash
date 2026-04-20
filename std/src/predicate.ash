-- Type predicate functions
-- Check the type of a value at runtime.

pub builtin fn is_int(value: a) -> Bool;
pub builtin fn is_string(value: a) -> Bool;
pub builtin fn is_bool(value: a) -> Bool;
pub builtin fn is_list(value: a) -> Bool;
pub builtin fn is_record(value: a) -> Bool;
pub builtin fn is_null(value: a) -> Bool;
