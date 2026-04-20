-- List operations
-- These are implemented as builtins in the runtime.

pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, item: a) -> List<a>;
pub builtin fn concat<a>(a: List<a>, b: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: Fn(a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, f: Fn(a) -> b) -> List<b>;
