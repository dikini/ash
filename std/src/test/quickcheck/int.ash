use test::quickcheck::context::{GenContext};
use test::quickcheck::strategy::{Strategy};

pub builtin fn gen(ctx: GenContext) -> Int;
pub builtin fn gen_small(ctx: GenContext) -> Int;
pub builtin fn gen_positive(ctx: GenContext) -> Int;
pub builtin fn gen_nonzero(ctx: GenContext) -> Int;
pub builtin fn shrink(value: Int) -> List<Int>;

pub fn ints() -> Strategy<Int> {
    Strategy { gen: gen, shrink: shrink }
}

pub fn small() -> Strategy<Int> {
    Strategy { gen: gen_small, shrink: shrink }
}

pub fn positive() -> Strategy<Int> {
    Strategy { gen: gen_positive, shrink: shrink }
}

pub fn nonzero() -> Strategy<Int> {
    Strategy { gen: gen_nonzero, shrink: shrink }
}

pub impl Arbitrary<Int> {
    arbitrary() = ints()
}
