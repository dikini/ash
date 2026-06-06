pub interface Applicative<F : * -> *> {
    pure(Int) -> F<Int>
    apply(F<Int -> Int>, F<Int>) -> F<Int>
}
