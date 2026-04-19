-- concat: declared with 2 params; the runtime implementation is variadic
-- (accepts any number of String args). Variadic surface syntax is not yet
-- supported, so this declaration intentionally constrains to 2 for type-checking.
pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;
