pub interface Monoid<A> {
    empty() -> A
    append(A, A) -> A
}

pub impl Monoid<String> {
    empty() = ""
    append(left, right) = string::concat(left, right)
}

pub impl <A : *> Monoid<List<A>> {
    empty() = []
    append(left, right) = list::concat(left, right)
}
