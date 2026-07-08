pub interface Comonad<W : * -> *> {
    extract(W<A>) -> A
    extend(W<A>, (W<A>) -> B) -> W<B>
}
