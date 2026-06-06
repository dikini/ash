pub interface Functor<F : * -> *> {
    map(F<Int>, Int -> Int) -> F<Int>
}

pub impl Functor<Option> {
    map(value, f) = option::map(value, f)
}

pub impl <E : *> Functor<Result<_, E>> {
    map(value, f) = result::map(value, f)
}

pub impl Functor<List> {
    map(value, f) = list::map(value, f)
}
