use algebra::functor::{Functor}

pub interface Applicative<F : * -> *> where F: Functor {
    pure(A) -> F<A>
    apply(F<A -> B>, F<A>) -> F<B>
}
