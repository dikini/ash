use test::quickcheck::context::{GenContext};
use test::quickcheck::strategy::{Strategy};

pub builtin fn gen<T>(ctx: GenContext) -> List<T>;
pub builtin fn gen_nonempty_int(ctx: GenContext) -> List<Int>;
pub builtin fn gen_sorted_int(ctx: GenContext) -> List<Int>;
pub builtin fn shrink<T>(value: List<T>) -> List<List<T>>;
pub builtin fn shrink_int_list(value: List<Int>) -> List<List<Int>>;

pub fn list_of<T>(element: Strategy<T>) -> Strategy<List<T>> {
    Strategy { gen: gen, shrink: shrink }
}

pub fn nonempty_ints() -> Strategy<List<Int>> {
    Strategy { gen: gen_nonempty_int, shrink: shrink_int_list }
}

pub fn sorted_ints() -> Strategy<List<Int>> {
    Strategy { gen: gen_sorted_int, shrink: shrink_int_list }
}
