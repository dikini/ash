-- List operations
-- These are implemented as builtins in the runtime.

use algebra::functor::{Functor}
use algebra::semigroup::{Semigroup}
use algebra::monoid::{Monoid}

pub impl Functor<List> {
    map(value, f) = list::map(value, f)
}

pub impl <A : *> Semigroup<List<A>> {
    append(left, right) = list::concat(left, right)
}

pub impl <A : *> Monoid<List<A>> {
    empty() = []
    append(left, right) = list::concat(left, right)
}

pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, item: a) -> List<a>;
pub builtin fn concat<a>(a: List<a>, b: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: (a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, f: (a) -> b) -> List<b>;
