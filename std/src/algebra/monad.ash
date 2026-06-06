pub interface Monad<M : * -> *> {
    unit(Int) -> M<Int>
    bind(M<Int>, Int -> M<Int>) -> M<Int>
}
