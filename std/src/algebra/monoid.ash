pub interface Monoid<A> {
    empty() -> A
    append(A, A) -> A
}
