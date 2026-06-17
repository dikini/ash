use test::quickcheck::context::{GenContext};
use test::quickcheck::strategy::{Strategy};

pub builtin fn gen(ctx: GenContext) -> Bool;
pub builtin fn shrink(value: Bool) -> List<Bool>;

pub fn bools() -> Strategy<Bool> {
    Strategy { gen: gen, shrink: shrink }
}

pub impl Arbitrary<Bool> {
    arbitrary() = bools()
}
