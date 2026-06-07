pub interface Comonad<W : * -> *> {
    extract(W<Int>) -> Int
    extend(W<Int>, W<Int> -> Int) -> W<Int>
}
