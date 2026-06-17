use test::quickcheck::context::{GenContext};
use test::quickcheck::strategy::{Strategy};

pub builtin fn gen(ctx: GenContext) -> String;
pub builtin fn gen_identifier(ctx: GenContext) -> String;
pub builtin fn shrink(value: String) -> List<String>;

pub fn strings() -> Strategy<String> {
    Strategy { gen: gen, shrink: shrink }
}

pub fn identifiers() -> Strategy<String> {
    Strategy { gen: gen_identifier, shrink: shrink }
}

pub impl Arbitrary<String> {
    arbitrary() = strings()
}
