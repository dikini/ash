-- concat: declared with 2 params; the runtime implementation is variadic
-- (accepts any number of String args). Variadic surface syntax is not yet
-- supported, so this declaration intentionally constrains to 2 for type-checking.

use algebra::semigroup::{Semigroup}
use algebra::monoid::{Monoid}

pub impl Semigroup<String> {
    append(left, right) = string::concat(left, right)
}

pub impl Monoid<String> {
    empty() = ""
    append(left, right) = string::concat(left, right)
}

pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;
pub builtin fn to_upper(s: String) -> String;
pub builtin fn to_lower(s: String) -> String;
pub builtin fn trim(s: String) -> String;
