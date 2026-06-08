use algebra::semigroup::{Semigroup}

pub interface Monoid<A> where A: Semigroup {
    empty() -> A
    append(A, A) -> A
}
