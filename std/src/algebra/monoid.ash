use algebra::semigroup::{Semigroup}
use algebra::eq::{Eq}

pub interface Monoid<A> where A: Semigroup {
    empty() -> A
    append(A, A) -> A
    law left_identity(a: A, eq: Eq<A>): eq.equiv(append(empty(), a), a)
    law right_identity(a: A, eq: Eq<A>): eq.equiv(append(a, empty()), a)
}
