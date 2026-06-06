pub interface Semigroup<A> {
    append(A, A) -> A
}

pub impl Semigroup<String> {
    append(left, right) = string::concat(left, right)
}

pub impl <A : *> Semigroup<List<A>> {
    append(left, right) = list::concat(left, right)
}
