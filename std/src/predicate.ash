-- Type predicate functions
-- Check the type of a value at runtime.

pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
