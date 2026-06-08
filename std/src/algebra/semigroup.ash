use algebra::eq::{Eq}

pub interface Semigroup<A> {
    append(A, A) -> A
    law associativity(a: A, b: A, c: A, eq: Eq<A>): eq.equiv(append(append(a, b), c), append(a, append(b, c)))
}
