pub interface Functor<F : * -> *> {
    map(F<A>, A -> B) -> F<B>
}
