pub interface Functor<F : * -> *> {
    map(F<Int>, Int -> Int) -> F<Int>
}
