pub interface Monoid<A> {
    empty() -> A
    append(A, A) -> A
}

pub fn concat_string(left: String, right: String) -> String {
    string::concat(left, right)
}

pub fn concat_list<A>(left: List<A>, right: List<A>) -> List<A> {
    list::concat(left, right)
}
