pub interface Functor<F : * -> *> {
    map(F<Int>, Int -> Int) -> F<Int>
}

pub fn map_option(value: Option<Int>, f: (Int) -> Int) -> Option<Int> {
    option::map(value, f)
}

pub fn map_result<E>(value: Result<Int, E>, f: (Int) -> Int) -> Result<Int, E> {
    result::map(value, f)
}

pub fn map_list(value: List<Int>, f: (Int) -> Int) -> List<Int> {
    list::map(value, f)
}
